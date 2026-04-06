/**
 * @jellylegsai/aether-sdk
 * 
 * Aether Blockchain SDK - TypeScript wrapper for Next.js API routes
 * All calls go directly to the Aether RPC endpoint.
 * 
 * Default RPC: http://127.0.0.1:8899 (configurable via AETHER_RPC env)
 */

import http from 'http';
import https from 'https';

export const DEFAULT_RPC_URL = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

// Re-export constants
export const DEFAULT_TIMEOUT_MS = 10000;
export const DEFAULT_RETRY_ATTEMPTS = 3;
export const DEFAULT_RETRY_DELAY_MS = 1000;
export const DEFAULT_BACKOFF_MULTIPLIER = 2;
export const DEFAULT_MAX_RETRY_DELAY_MS = 30000;
export const DEFAULT_RATE_LIMIT_RPS = 10;
export const DEFAULT_RATE_LIMIT_BURST = 20;
export const DEFAULT_CIRCUIT_BREAKER_THRESHOLD = 5;
export const DEFAULT_CIRCUIT_BREAKER_RESET_MS = 60000;

// Custom error types
export class AetherSDKError extends Error {
  code: string;
  details: Record<string, unknown>;
  timestamp: string;
  constructor(message: string, code: string, details = {}) {
    super(message);
    this.name = 'AetherSDKError';
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();
  }
}

export class NetworkTimeoutError extends Error {
  code: string;
  details: Record<string, unknown>;
  constructor(message: string, details = {}) {
    super(message);
    this.name = 'NetworkTimeoutError';
    this.code = 'ETIMEDOUT';
    this.details = details;
  }
}

export class RPCError extends Error {
  code: string | number;
  details: Record<string, unknown>;
  constructor(message: string, details: { code?: string | number } = {}) {
    super(message);
    this.name = 'RPCError';
    this.code = details.code ?? 'RPC_ERROR';
    this.details = details;
  }
}

export class CircuitBreakerOpenError extends Error {
  details: Record<string, unknown>;
  constructor(message: string, details = {}) {
    super(message);
    this.name = 'CircuitBreakerOpenError';
    this.details = details;
  }
}

// TokenBucketRateLimiter (minimal implementation)
class TokenBucketRateLimiter {
  private tokens: number;
  private lastRefill: number;
  private readonly refillRate: number;
  private readonly burstCapacity: number;

  constructor(rps: number, burst: number) {
    this.refillRate = rps;
    this.burstCapacity = burst;
    this.tokens = burst;
    this.lastRefill = Date.now();
  }

  async acquire(): Promise<void> {
    this.refill();
    if (this.tokens >= 1) {
      this.tokens--;
      return;
    }
    const waitTime = (1 - this.tokens) / this.refillRate * 1000;
    await new Promise(resolve => setTimeout(resolve, waitTime));
    this.refill();
    this.tokens--;
  }

  private refill(): void {
    const now = Date.now();
    const elapsed = (now - this.lastRefill) / 1000;
    this.tokens = Math.min(this.burstCapacity, this.tokens + elapsed * this.refillRate);
    this.lastRefill = now;
  }
}

// CircuitBreaker (minimal implementation)
class CircuitBreaker {
  private failures = 0;
  private lastFailureTime = 0;
  private state: 'closed' | 'open' | 'half-open' = 'closed';
  private readonly threshold: number;
  private readonly resetMs: number;

  constructor(threshold: number, resetMs: number) {
    this.threshold = threshold;
    this.resetMs = resetMs;
  }

  canExecute(): boolean {
    if (this.state === 'closed') return true;
    if (this.state === 'open') {
      if (Date.now() - this.lastFailureTime > this.resetMs) {
        this.state = 'half-open';
        return true;
      }
      return false;
    }
    return true;
  }

  recordSuccess(): void {
    this.failures = 0;
    this.state = 'closed';
  }

  recordFailure(): void {
    this.failures++;
    this.lastFailureTime = Date.now();
    if (this.failures >= this.threshold) {
      this.state = 'open';
    }
  }

  getState() {
    return { state: this.state, failures: this.failures, nextAttempt: this.lastFailureTime + this.resetMs };
  }
}

interface AetherClientOptions {
  rpcUrl?: string;
  timeoutMs?: number;
  retryAttempts?: number;
  retryDelayMs?: number;
  backoffMultiplier?: number;
  maxRetryDelayMs?: number;
  rateLimitRps?: number;
  rateLimitBurst?: number;
  circuitBreakerThreshold?: number;
  circuitBreakerResetMs?: number;
}

interface RPCResponse {
  jsonrpc: string;
  id: number;
  result?: unknown;
  error?: { message: string; code?: number };
}

/**
 * AetherClient - Main SDK class for interacting with Aether blockchain
 */
export class AetherClient {
  private rpcUrl: string;
  private timeoutMs: number;
  private retryAttempts: number;
  private retryDelayMs: number;
  private backoffMultiplier: number;
  private maxRetryDelayMs: number;
  private rateLimiter: TokenBucketRateLimiter;
  private circuitBreaker: CircuitBreaker;
  private protocol: string;
  private hostname: string;
  private port: number;
  
  readonly stats = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    retriedRequests: 0,
    rateLimitedRequests: 0,
    circuitBreakerBlocked: 0,
  };

  constructor(options: AetherClientOptions = {}) {
    this.rpcUrl = options.rpcUrl || process.env.AETHER_RPC || DEFAULT_RPC_URL;
    this.timeoutMs = options.timeoutMs || DEFAULT_TIMEOUT_MS;
    this.retryAttempts = options.retryAttempts ?? DEFAULT_RETRY_ATTEMPTS;
    this.retryDelayMs = options.retryDelayMs ?? DEFAULT_RETRY_DELAY_MS;
    this.backoffMultiplier = options.backoffMultiplier ?? DEFAULT_BACKOFF_MULTIPLIER;
    this.maxRetryDelayMs = options.maxRetryDelayMs ?? DEFAULT_MAX_RETRY_DELAY_MS;
    
    this.rateLimiter = new TokenBucketRateLimiter(
      options.rateLimitRps ?? DEFAULT_RATE_LIMIT_RPS,
      options.rateLimitBurst ?? DEFAULT_RATE_LIMIT_BURST
    );
    
    this.circuitBreaker = new CircuitBreaker(
      options.circuitBreakerThreshold ?? DEFAULT_CIRCUIT_BREAKER_THRESHOLD,
      options.circuitBreakerResetMs ?? DEFAULT_CIRCUIT_BREAKER_RESET_MS
    );

    const url = new URL(this.rpcUrl);
    this.protocol = url.protocol;
    this.hostname = url.hostname;
    this.port = parseInt(url.port) || (this.protocol === 'https:' ? 443 : 80);
  }

  private calculateDelay(attempt: number): number {
    const baseDelay = this.retryDelayMs * Math.pow(this.backoffMultiplier, attempt);
    const jitter = Math.random() * 100;
    return Math.min(baseDelay + jitter, this.maxRetryDelayMs);
  }

  private isRetryableError(error: { code?: string; message?: string; statusCode?: number }): boolean {
    if (!error) return false;
    if (error.code === 'ECONNREFUSED') return true;
    if (error.code === 'ENOTFOUND') return true;
    if (error.code === 'ETIMEDOUT') return true;
    if (error.code === 'ECONNRESET') return true;
    if (error.code === 'EPIPE') return true;
    if (error.message?.includes('timeout')) return true;
    if (error.statusCode !== undefined && error.statusCode >= 500) return true;
    if (error.statusCode === 429) return true;
    if (error.message?.includes('rate limit') || error.message?.includes('temporarily unavailable')) return true;
    return false;
  }

  private async executeWithRetry<T>(operation: () => Promise<T>, operationName: string): Promise<T> {
    if (!this.circuitBreaker.canExecute()) {
      this.stats.circuitBreakerBlocked++;
      const state = this.circuitBreaker.getState();
      const waitTime = Math.ceil((state.nextAttempt - Date.now()) / 1000);
      throw new CircuitBreakerOpenError(
        `Circuit breaker is OPEN. Too many failures. Retry in ${waitTime}s.`,
        { circuitBreakerState: state, operation: operationName }
      );
    }

    await this.rateLimiter.acquire();

    let lastError: Error | null = null;
    for (let attempt = 0; attempt < this.retryAttempts; attempt++) {
      this.stats.totalRequests++;
      try {
        const result = await operation();
        this.circuitBreaker.recordSuccess();
        this.stats.successfulRequests++;
        return result;
      } catch (error: any) {
        lastError = error;
        if (!this.isRetryableError(error)) {
          this.circuitBreaker.recordFailure();
          this.stats.failedRequests++;
          break;
        }
        this.stats.retriedRequests++;
        this.circuitBreaker.recordFailure();
        if (attempt === this.retryAttempts - 1) {
          this.stats.failedRequests++;
          break;
        }
        const delay = this.calculateDelay(attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    throw lastError || new Error(`Operation ${operationName} failed`);
  }

  private async rpcCall(method: string, params: unknown[] = []): Promise<unknown> {
    return this.executeWithRetry(async () => {
      return new Promise((resolve, reject) => {
        const body = JSON.stringify({ jsonrpc: '2.0', id: 1, method, params });
        const lib = this.protocol === 'https:' ? https : http;
        
        const req = lib.request({
          hostname: this.hostname,
          port: this.port,
          path: '/',
          method: 'POST',
          timeout: this.timeoutMs,
          headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(body),
          },
        }, (res) => {
          let data = '';
          res.on('data', (chunk) => data += chunk);
          res.on('end', () => {
            try {
              const parsed: RPCResponse = JSON.parse(data);
              if (parsed.error) {
                const err = new Error(parsed.error.message || JSON.stringify(parsed.error));
                (err as any).statusCode = res.statusCode;
                reject(err);
              } else {
                resolve(parsed.result);
              }
            } catch (e) {
              resolve({ raw: data });
            }
          });
        });
        
        req.on('error', reject);
        req.on('timeout', () => {
          req.destroy();
          const err = new Error(`Request timeout after ${this.timeoutMs}ms`);
          (err as any).code = 'ETIMEDOUT';
          reject(err);
        });
        
        req.write(body);
        req.end();
      });
    }, `rpc.${method}`);
  }

  // Public API methods

  async getSlot(): Promise<number> {
    const result = await this.rpcCall('getSlot', []);
    return result as number;
  }

  async getBlockHeight(): Promise<number> {
    const result = await this.rpcCall('getBlockHeight', []);
    return result as number;
  }

  async getAccountInfo(address: string): Promise<Record<string, unknown> | null> {
    const result = await this.rpcCall('getAccountInfo', [address, { encoding: 'json' }]);
    return result as Record<string, unknown> | null;
  }

  async getBalance(address: string): Promise<number> {
    const result = await this.rpcCall('getBalance', [address]);
    return (result as { value: number }).value;
  }

  async getEpochInfo(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getEpochInfo', []);
    return result as Record<string, unknown>;
  }

  async getTransaction(signature: string): Promise<Record<string, unknown> | null> {
    const result = await this.rpcCall('getTransaction', [signature, { encoding: 'json', maxSupportedTransactionVersion: 0 }]);
    return result as Record<string, unknown> | null;
  }

  async getRecentBlockhash(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getRecentBlockhash', [{ encoding: 'json' }]);
    return result as Record<string, unknown>;
  }

  async getClusterPeers(): Promise<unknown[]> {
    const result = await this.rpcCall('getClusterPeers', []);
    return result as unknown[];
  }

  async getValidators(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getValidators', []);
    return result as Record<string, unknown>;
  }

  async getSupply(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getSupply', [{ encoding: 'json' }]);
    return result as Record<string, unknown>;
  }

  async getHealth(): Promise<string> {
    const result = await this.rpcCall('getHealth', []);
    return result as string;
  }

  async getVersion(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getVersion', []);
    return result as Record<string, unknown>;
  }

  async getTPS(): Promise<number> {
    const result = await this.rpcCall('getCurrentTPS', []);
    return result as number;
  }

  async getFees(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getFees', []);
    return result as Record<string, unknown>;
  }

  async getSlotProduction(): Promise<Record<string, unknown>> {
    const result = await this.rpcCall('getSlotProduction', []);
    return result as Record<string, unknown>;
  }

  async getStakePositions(address: string): Promise<unknown[]> {
    // Attempt to get stake accounts for the address
    // Falls back to empty array if stake program not available
    try {
      const result = await this.rpcCall('getStakeAccounts', [address]);
      return result as unknown[] || [];
    } catch {
      // Stake program may not be active - return mock positions for demo
      return [];
    }
  }

  async getRewards(address: string): Promise<Record<string, unknown>> {
    try {
      const result = await this.rpcCall('getRewards', [address]);
      return result as Record<string, unknown>;
    } catch {
      return { total_rewards: 0, claimable_rewards: 0, claimed_rewards: 0, epoch: 0 };
    }
  }

  async getValidatorAPY(validatorAddr: string): Promise<number> {
    try {
      const result = await this.rpcCall('getValidatorAPY', [validatorAddr]);
      return result as number;
    } catch {
      return 0;
    }
  }

  async getRecentTransactions(address: string, limit = 20): Promise<unknown[]> {
    try {
      const result = await this.rpcCall('getRecentTransactions', [address, limit]);
      return result as unknown[];
    } catch {
      return [];
    }
  }

  async getTransactionHistory(address: string, limit = 20): Promise<unknown[]> {
    try {
      const result = await this.rpcCall('getTransactionHistory', [address, limit]);
      return result as unknown[];
    } catch {
      return [];
    }
  }

  async getTokenAccounts(address: string): Promise<unknown[]> {
    try {
      const result = await this.rpcCall('getTokenAccounts', [address, { encoding: 'json' }]);
      return result as unknown[];
    } catch {
      return [];
    }
  }

  async getStakeAccounts(address: string): Promise<unknown[]> {
    try {
      const result = await this.rpcCall('getStakeAccounts', [address]);
      return result as unknown[];
    } catch {
      return [];
    }
  }

  async ping(): Promise<{ ok: boolean; latency?: number; rpc: string; error?: string }> {
    const start = Date.now();
    try {
      await this.getSlot();
      return { ok: true, latency: Date.now() - start, rpc: this.rpcUrl };
    } catch (err: any) {
      return { ok: false, error: err.message, rpc: this.rpcUrl };
    }
  }
}

export default AetherClient;
