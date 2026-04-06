/**
 * @jellylegsai/aether-sdk
 * 
 * Official Aether Blockchain SDK - Real HTTP RPC calls to Aether nodes
 * No stubs, no mocks - every function makes actual blockchain calls
 * 
 * Features:
 * - Retry logic with exponential backoff
 * - Rate limiting with token bucket algorithm
 * - Enhanced error handling for network timeouts and RPC failures
 * - Circuit breaker for repeated failures
 * 
 * Default RPC: http://127.0.0.1:8899 (configurable via constructor or AETHER_RPC env)
 */

const http = require('http');
const https = require('https');
const { rpcGet, rpcPost } = require('./rpc');

// Default configuration
const DEFAULT_RPC_URL = 'http://127.0.0.1:8899';
const DEFAULT_TIMEOUT_MS = 10000;

// Retry configuration
const DEFAULT_RETRY_ATTEMPTS = 3;
const DEFAULT_RETRY_DELAY_MS = 1000;
const DEFAULT_BACKOFF_MULTIPLIER = 2;
const DEFAULT_MAX_RETRY_DELAY_MS = 30000;

// Rate limiting configuration
const DEFAULT_RATE_LIMIT_RPS = 10; // Requests per second
const DEFAULT_RATE_LIMIT_BURST = 20; // Burst capacity

// Circuit breaker configuration
const DEFAULT_CIRCUIT_BREAKER_THRESHOLD = 5; // Failures before opening
const DEFAULT_CIRCUIT_BREAKER_RESET_MS = 60000; // Reset after 60s

/**
 * Custom error types for better error handling
 */
class AetherSDKError extends Error {
  constructor(message, code, details = {}) {
    super(message);
    this.name = 'AetherSDKError';
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();
  }
}

class NetworkTimeoutError extends AetherSDKError {
  constructor(message, details = {}) {
    super(message, 'NETWORK_TIMEOUT', details);
    this.name = 'NetworkTimeoutError';
  }
}

class RPCError extends AetherSDKError {
  constructor(message, details = {}) {
    super(message, 'RPC_ERROR', details);
    this.name = 'RPCError';
  }
}

class RateLimitError extends AetherSDKError {
  constructor(message, details = {}) {
    super(message, 'RATE_LIMIT', details);
    this.name = 'RateLimitError';
  }
}

class CircuitBreakerOpenError extends AetherSDKError {
  constructor(message, details = {}) {
    super(message, 'CIRCUIT_BREAKER_OPEN', details);
    this.name = 'CircuitBreakerOpenError';
  }
}

/**
 * Token bucket rate limiter
 */
class TokenBucketRateLimiter {
  constructor(rps = DEFAULT_RATE_LIMIT_RPS, burst = DEFAULT_RATE_LIMIT_BURST) {
    this.rps = rps;
    this.burst = burst;
    this.tokens = burst;
    this.lastRefill = Date.now();
    this.queue = [];
    this.refillInterval = setInterval(() => this.refill(), 1000 / rps);
  }

  refill() {
    const now = Date.now();
    const timePassed = (now - this.lastRefill) / 1000;
    const tokensToAdd = timePassed * this.rps;
    this.tokens = Math.min(this.burst, this.tokens + tokensToAdd);
    this.lastRefill = now;
    this.processQueue();
  }

  processQueue() {
    while (this.queue.length > 0 && this.tokens >= 1) {
      const { resolve, reject, tokens } = this.queue.shift();
      if (this.tokens >= tokens) {
        this.tokens -= tokens;
        resolve();
      } else {
        this.queue.unshift({ resolve, reject, tokens });
        break;
      }
    }
  }

  async acquire(tokens = 1) {
    return new Promise((resolve, reject) => {
      if (this.tokens >= tokens) {
        this.tokens -= tokens;
        resolve();
      } else {
        this.queue.push({ resolve, reject, tokens });
      }
    });
  }

  destroy() {
    if (this.refillInterval) {
      clearInterval(this.refillInterval);
      this.refillInterval = null;
    }
  }
}

/**
 * Circuit breaker for handling repeated failures
 */
class CircuitBreaker {
  constructor(threshold = DEFAULT_CIRCUIT_BREAKER_THRESHOLD, resetTimeoutMs = DEFAULT_CIRCUIT_BREAKER_RESET_MS) {
    this.threshold = threshold;
    this.resetTimeoutMs = resetTimeoutMs;
    this.failureCount = 0;
    this.state = 'CLOSED'; // CLOSED, OPEN, HALF_OPEN
    this.nextAttempt = 0;
  }

  canExecute() {
    if (this.state === 'CLOSED') return true;
    if (this.state === 'OPEN') {
      if (Date.now() >= this.nextAttempt) {
        this.state = 'HALF_OPEN';
        return true;
      }
      return false;
    }
    return this.state === 'HALF_OPEN';
  }

  recordSuccess() {
    this.failureCount = 0;
    this.state = 'CLOSED';
  }

  recordFailure() {
    this.failureCount++;
    if (this.failureCount >= this.threshold) {
      this.state = 'OPEN';
      this.nextAttempt = Date.now() + this.resetTimeoutMs;
    }
  }

  getState() {
    return {
      state: this.state,
      failureCount: this.failureCount,
      nextAttempt: this.state === 'OPEN' ? this.nextAttempt : null,
    };
  }
}

/**
 * Aether SDK Client
 * Real blockchain interface layer - every method makes actual HTTP RPC calls
 * 
 * Includes:
 * - Retry logic with exponential backoff
 * - Rate limiting
 * - Circuit breaker for resilience
 * - Enhanced error handling
 */
class AetherClient {
  constructor(options = {}) {
    this.rpcUrl = options.rpcUrl || process.env.AETHER_RPC || DEFAULT_RPC_URL;
    this.timeoutMs = options.timeoutMs || DEFAULT_TIMEOUT_MS;
    
    // Retry configuration
    this.retryAttempts = options.retryAttempts ?? DEFAULT_RETRY_ATTEMPTS;
    this.retryDelayMs = options.retryDelayMs ?? DEFAULT_RETRY_DELAY_MS;
    this.backoffMultiplier = options.backoffMultiplier ?? DEFAULT_BACKOFF_MULTIPLIER;
    this.maxRetryDelayMs = options.maxRetryDelayMs ?? DEFAULT_MAX_RETRY_DELAY_MS;
    
    // Rate limiting
    this.rateLimiter = new TokenBucketRateLimiter(
      options.rateLimitRps ?? DEFAULT_RATE_LIMIT_RPS,
      options.rateLimitBurst ?? DEFAULT_RATE_LIMIT_BURST
    );
    
    // Circuit breaker
    this.circuitBreaker = new CircuitBreaker(
      options.circuitBreakerThreshold ?? DEFAULT_CIRCUIT_BREAKER_THRESHOLD,
      options.circuitBreakerResetMs ?? DEFAULT_CIRCUIT_BREAKER_RESET_MS
    );

    // Parse RPC URL
    const url = new URL(this.rpcUrl);
    this.protocol = url.protocol;
    this.hostname = url.hostname;
    this.port = url.port || (this.protocol === 'https:' ? 443 : 80);
    
    // Request stats
    this.stats = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      retriedRequests: 0,
      rateLimitedRequests: 0,
      circuitBreakerBlocked: 0,
    };
  }

  /**
   * Calculate delay for exponential backoff with jitter
   */
  _calculateDelay(attempt) {
    const baseDelay = this.retryDelayMs * Math.pow(this.backoffMultiplier, attempt);
    const jitter = Math.random() * 100; // Add up to 100ms jitter
    const delay = Math.min(baseDelay + jitter, this.maxRetryDelayMs);
    return delay;
  }

  /**
   * Check if error is retryable
   */
  _isRetryableError(error) {
    if (!error) return false;
    
    // Network errors
    if (error.code === 'ECONNREFUSED') return true;
    if (error.code === 'ENOTFOUND') return true;
    if (error.code === 'ETIMEDOUT') return true;
    if (error.code === 'ECONNRESET') return true;
    if (error.code === 'EPIPE') return true;
    
    // Timeout errors
    if (error.message && error.message.includes('timeout')) return true;
    
    // HTTP 5xx errors (server errors)
    if (error.statusCode >= 500) return true;
    if (error.statusCode === 429) return true; // Rate limit - retry with backoff
    
    // RPC errors that might be transient
    if (error.message && (
      error.message.includes('rate limit') ||
      error.message.includes('rate_limit') ||
      error.message.includes('too many requests') ||
      error.message.includes('temporarily unavailable') ||
      error.message.includes('service unavailable')
    )) return true;
    
    return false;
  }

  /**
   * Execute function with retry logic and rate limiting
   */
  async _executeWithRetry(operation, operationName) {
    // Check circuit breaker
    if (!this.circuitBreaker.canExecute()) {
      this.stats.circuitBreakerBlocked++;
      const state = this.circuitBreaker.getState();
      const waitTime = Math.ceil((state.nextAttempt - Date.now()) / 1000);
      throw new CircuitBreakerOpenError(
        `Circuit breaker is OPEN. Too many failures. Retry in ${waitTime}s.`,
        { circuitBreakerState: state, operation: operationName }
      );
    }

    // Wait for rate limit token
    await this.rateLimiter.acquire();

    let lastError = null;
    
    for (let attempt = 0; attempt < this.retryAttempts; attempt++) {
      this.stats.totalRequests++;
      
      try {
        const result = await operation();
        this.circuitBreaker.recordSuccess();
        this.stats.successfulRequests++;
        return result;
      } catch (error) {
        lastError = error;
        
        // Don't retry if it's not a retryable error
        if (!this._isRetryableError(error)) {
          this.circuitBreaker.recordFailure();
          this.stats.failedRequests++;
          break;
        }
        
        this.stats.retriedRequests++;
        this.circuitBreaker.recordFailure();
        
        // If this was the last attempt, throw the error
        if (attempt === this.retryAttempts - 1) {
          this.stats.failedRequests++;
          break;
        }
        
        // Calculate and apply backoff delay
        const delay = this._calculateDelay(attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    
    // All retries exhausted - classify and throw error
    throw this._classifyError(lastError, operationName);
  }

  /**
   * Classify error into specific error types
   */
  _classifyError(error, operationName) {
    if (!error) {
      return new AetherSDKError('Unknown error occurred', 'UNKNOWN_ERROR', { operation: operationName });
    }
    
    // Already classified
    if (error instanceof AetherSDKError) {
      return error;
    }
    
    // Timeout errors
    if (error.message && (
      error.message.includes('timeout') ||
      error.code === 'ETIMEDOUT'
    )) {
      return new NetworkTimeoutError(
        `Network timeout during ${operationName}: ${error.message}`,
        { 
          originalError: error.message,
          code: error.code,
          operation: operationName,
          rpcUrl: this.rpcUrl,
        }
      );
    }
    
    // Connection errors
    if (error.code === 'ECONNREFUSED' || error.code === 'ENOTFOUND') {
      return new AetherSDKError(
        `Cannot connect to RPC endpoint during ${operationName}: ${error.message}`,
        'CONNECTION_ERROR',
        {
          originalError: error.message,
          code: error.code,
          operation: operationName,
          rpcUrl: this.rpcUrl,
        }
      );
    }
    
    // RPC-specific errors
    if (error.message && (
      error.message.includes('RPC') ||
      error.message.includes('rpc') ||
      error.statusCode
    )) {
      return new RPCError(
        `RPC error during ${operationName}: ${error.message}`,
        {
          originalError: error.message,
          code: error.code || error.statusCode,
          operation: operationName,
          rpcUrl: this.rpcUrl,
        }
      );
    }
    
    // Generic error
    return new AetherSDKError(
      `Error during ${operationName}: ${error.message}`,
      'SDK_ERROR',
      {
        originalError: error.message,
        code: error.code,
        operation: operationName,
        rpcUrl: this.rpcUrl,
      }
    );
  }

  /**
   * Internal: Make HTTP GET request to RPC endpoint
   */
  _httpGet(path, timeoutMs = this.timeoutMs) {
    return new Promise((resolve, reject) => {
      const lib = this.protocol === 'https:' ? https : http;
      const req = lib.request({
        hostname: this.hostname,
        port: this.port,
        path: path,
        method: 'GET',
        timeout: timeoutMs,
        headers: { 'Content-Type': 'application/json' },
      }, (res) => {
        let data = '';
        res.on('data', (chunk) => data += chunk);
        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);
            if (parsed.error) {
              const err = new Error(parsed.error.message || JSON.stringify(parsed.error));
              err.statusCode = res.statusCode;
              err.responseData = parsed;
              reject(err);
            } else {
              resolve(parsed);
            }
          } catch (e) {
            resolve({ raw: data });
          }
        });
      });
      req.on('error', reject);
      req.on('timeout', () => {
        req.destroy();
        const err = new Error(`Request timeout after ${timeoutMs}ms`);
        err.code = 'ETIMEDOUT';
        reject(err);
      });
      req.end();
    });
  }

  /**
   * Internal: Make HTTP POST request to RPC endpoint
   */
  _httpPost(path, body = {}, timeoutMs = this.timeoutMs) {
    return new Promise((resolve, reject) => {
      const lib = this.protocol === 'https:' ? https : http;
      const bodyStr = JSON.stringify(body);
      const req = lib.request({
        hostname: this.hostname,
        port: this.port,
        path: path,
        method: 'POST',
        timeout: timeoutMs,
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(bodyStr),
        },
      }, (res) => {
        let data = '';
        res.on('data', (chunk) => data += chunk);
        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);
            if (parsed.error) {
              const err = new Error(parsed.error.message || JSON.stringify(parsed.error));
              err.statusCode = res.statusCode;
              err.responseData = parsed;
              reject(err);
            } else {
              resolve(parsed);
            }
          } catch (e) {
            resolve({ raw: data });
          }
        });
      });
      req.on('error', reject);
      req.on('timeout', () => {
        req.destroy();
        const err = new Error(`Request timeout after ${timeoutMs}ms`);
        err.code = 'ETIMEDOUT';
        reject(err);
      });
      req.write(bodyStr);
      req.end();
    });
  }

  // ============================================================
  // Core RPC Methods - Real blockchain calls with retry & rate limiting
  // ============================================================

  /**
   * Get current slot number
   * RPC: GET /v1/slot
   * 
   * @returns {Promise<number>} Current slot number
   */
  async getSlot() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/slot');
        return result.slot !== undefined ? result.slot : result;
      },
      'getSlot'
    );
  }

  /**
   * Get current block height
   * RPC: GET /v1/blockheight
   * 
   * @returns {Promise<number>} Current block height
   */
  async getBlockHeight() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/blockheight');
        return result.blockHeight !== undefined ? result.blockHeight : result;
      },
      'getBlockHeight'
    );
  }

  /**
   * Get account info including balance
   * RPC: GET /v1/account/<address>
   * 
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Object>} Account info: { lamports, owner, data, rent_epoch }
   */
  async getAccountInfo(address) {
    if (!address) {
      throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    }
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/account/${address}`);
        return result;
      },
      'getAccountInfo'
    );
  }

  /**
   * Alias for getAccountInfo
   * @param {string} address - Account address
   * @returns {Promise<Object>} Account info
   */
  async getAccount(address) {
    return this.getAccountInfo(address);
  }

  /**
   * Get balance in lamports
   * RPC: GET /v1/account/<address>
   * 
   * @param {string} address - Account public key (base58)
   * @returns {Promise<number>} Balance in lamports
   */
  async getBalance(address) {
    const account = await this.getAccountInfo(address);
    return account.lamports !== undefined ? account.lamports : 0;
  }

  /**
   * Get epoch info
   * RPC: GET /v1/epoch
   * 
   * @returns {Promise<Object>} Epoch info: { epoch, slotIndex, slotsInEpoch, absoluteSlot }
   */
  async getEpochInfo() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/epoch');
        return result;
      },
      'getEpochInfo'
    );
  }

  /**
   * Get transaction by signature
   * RPC: GET /v1/transaction/<signature>
   * 
   * @param {string} signature - Transaction signature (base58)
   * @returns {Promise<Object>} Transaction details
   */
  async getTransaction(signature) {
    if (!signature) {
      throw new AetherSDKError('Transaction signature is required', 'VALIDATION_ERROR');
    }
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/transaction/${signature}`);
        return result;
      },
      'getTransaction'
    );
  }

  /**
   * Submit a signed transaction
   * RPC: POST /v1/transaction
   * 
   * @param {Object} tx - Signed transaction object
   * @param {string} tx.signature - Transaction signature (base58)
   * @param {string} tx.signer - Signer public key (base58)
   * @param {string} tx.tx_type - Transaction type
   * @param {Object} tx.payload - Transaction payload
   * @returns {Promise<Object>} Transaction receipt: { signature, slot, confirmed }
   */
  async sendTransaction(tx) {
    if (!tx || !tx.signature) {
      throw new AetherSDKError('Transaction with signature is required', 'VALIDATION_ERROR');
    }
    return this._executeWithRetry(
      async () => {
        const result = await this._httpPost('/v1/transaction', tx);
        return result;
      },
      'sendTransaction'
    );
  }

  /**
   * Get recent blockhash for transaction signing
   * RPC: GET /v1/recent-blockhash
   * 
   * @returns {Promise<Object>} { blockhash, lastValidBlockHeight }
   */
  async getRecentBlockhash() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/recent-blockhash');
        return result;
      },
      'getRecentBlockhash'
    );
  }

  /**
   * Get network peers
   * RPC: GET /v1/peers
   * 
   * @returns {Promise<Array>} List of peer node addresses
   */
  async getClusterPeers() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/peers');
        return Array.isArray(result) ? result : (result.peers || []);
      },
      'getClusterPeers'
    );
  }

  /**
   * Get validator info
   * RPC: GET /v1/validators
   * 
   * @returns {Promise<Array>} List of validators with stake, commission, etc.
   */
  async getValidators() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/validators');
        return Array.isArray(result) ? result : (result.validators || []);
      },
      'getValidators'
    );
  }

  /**
   * Get supply info
   * RPC: GET /v1/supply
   * 
   * @returns {Promise<Object>} Supply info: { total, circulating, nonCirculating }
   */
  async getSupply() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/supply');
        return result;
      },
      'getSupply'
    );
  }

  /**
   * Get health status
   * RPC: GET /v1/health
   * 
   * @returns {Promise<string>} 'ok' if node is healthy
   */
  async getHealth() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/health');
        return result.status || result;
      },
      'getHealth'
    );
  }

  /**
   * Get version info
   * RPC: GET /v1/version
   * 
   * @returns {Promise<Object>} Version info: { aetherCore, featureSet }
   */
  async getVersion() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/version');
        return result;
      },
      'getVersion'
    );
  }

  /**
   * Get TPS (transactions per second)
   * RPC: GET /v1/tps
   * 
   * @returns {Promise<number>} Current TPS
   */
  async getTPS() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/tps');
        return result.tps ?? result.tps_avg ?? result.transactions_per_second ?? null;
      },
      'getTPS'
    );
  }

  /**
   * Get fee estimates
   * RPC: GET /v1/fees
   * 
   * @returns {Promise<Object>} Fee info
   */
  async getFees() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet('/v1/fees');
        return result;
      },
      'getFees'
    );
  }

  /**
   * Get slot production stats
   * RPC: POST /v1/slot_production
   * 
   * @returns {Promise<Object>} Slot production stats
   */
  async getSlotProduction() {
    return this._executeWithRetry(
      async () => {
        const result = await this._httpPost('/v1/slot_production', {});
        return result;
      },
      'getSlotProduction'
    );
  }

  /**
   * Get stake positions for an address
   * RPC: GET /v1/stake/<address>
   * 
   * @param {string} address - Account address
   * @returns {Promise<Array>} List of stake positions
   */
  async getStakePositions(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/stake/${address}`);
        return result.delegations ?? result.stakes ?? result ?? [];
      },
      'getStakePositions'
    );
  }

  /**
   * Get rewards for an address
   * RPC: GET /v1/rewards/<address>
   * 
   * @param {string} address - Account address
   * @returns {Promise<Object>} Rewards info
   */
  async getRewards(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/rewards/${address}`);
        return result;
      },
      'getRewards'
    );
  }

  /**
   * Get validator APY
   * RPC: GET /v1/validator/<address>/apy
   * 
   * @param {string} validatorAddr - Validator address
   * @returns {Promise<Object>} APY info
   */
  async getValidatorAPY(validatorAddr) {
    if (!validatorAddr) throw new AetherSDKError('Validator address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/validator/${validatorAddr}/apy`);
        return result;
      },
      'getValidatorAPY'
    );
  }

  /**
   * Get recent transactions for an address
   * RPC: GET /v1/transactions/<address>?limit=<n>
   *
   * @param {string} address - Account address
   * @param {number} limit - Max transactions to return
   * @returns {Promise<Array>} List of recent transactions
   */
  async getRecentTransactions(address, limit = 20) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/transactions/${address}?limit=${limit}`);
        return result.transactions ?? result ?? [];
      },
      'getRecentTransactions'
    );
  }

  /**
   * Get transaction history with signatures for an address
   * RPC: POST /v1/transactions/history (or GET /v1/transactions/<address>?limit=<n>)
   *
   * @param {string} address - Account address
   * @param {number} limit - Max transactions to return
   * @returns {Promise<Object>} Transaction history with signatures and details
   */
  async getTransactionHistory(address, limit = 20) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    
    // First get signatures
    const sigResult = await this._executeWithRetry(
      async () => {
        const result = await this._httpPost('/v1/transactions/history', { address, limit });
        if (result.error) {
          throw new RPCError(result.error.message || result.error, { result });
        }
        return result;
      },
      'getTransactionHistory.signatures'
    );
    
    const signatures = sigResult.signatures || sigResult.result || [];
    
    // Fetch full transaction details for each signature (up to 10 at a time)
    const BATCH = 10;
    const txs = [];
    for (let i = 0; i < signatures.length; i += BATCH) {
      const batch = signatures.slice(i, i + BATCH);
      const batchPromises = batch.map(sig => 
        this.getTransaction(sig.signature || sig).catch(() => null)
      );
      const batchResults = await Promise.all(batchPromises);
      txs.push(...batchResults.filter(Boolean));
    }
    
    return {
      signatures: signatures,
      transactions: txs,
      address: address,
    };
  }

  /**
   * Get all SPL token accounts for a wallet address
   * RPC: GET /v1/tokens/<address>
   *
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Array>} List of token accounts with mint, amount, decimals
   */
  async getTokenAccounts(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/tokens/${address}`);
        return result.tokens ?? result.accounts ?? result ?? [];
      },
      'getTokenAccounts'
    );
  }

  /**
   * Get all stake accounts for a wallet address
   * RPC: GET /v1/stake-accounts/<address>
   *
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Array>} List of stake accounts
   */
  async getStakeAccounts(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/stake-accounts/${address}`);
        return result.stake_accounts ?? result.delegations ?? result ?? [];
      },
      'getStakeAccounts'
    );
  }

  // ============================================================
  // Transaction Helpers - Build and send real transactions
  // ============================================================

  /**
   * Build and send a transfer transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.from - Sender address (base58)
   * @param {string} params.to - Recipient address (base58)
   * @param {number} params.amount - Amount in lamports
   * @param {number} params.nonce - Nonce for replay protection
   * @param {Function} params.signFn - Function to sign the transaction (receives tx object, returns signature)
   * @returns {Promise<Object>} Transaction receipt
   */
  async transfer({ from, to, amount, nonce, signFn }) {
    if (!from || !to || !amount === undefined || nonce === undefined) {
      throw new AetherSDKError('from, to, amount, and nonce are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required (function to sign the transaction)', 'VALIDATION_ERROR');
    }

    // Get recent blockhash (real RPC call)
    const { blockhash } = await this.getRecentBlockhash();

    // Build transaction payload
    const tx = {
      signature: '', // Will be filled after signing
      signer: from,
      tx_type: 'Transfer',
      payload: {
        recipient: to,
        amount: BigInt(amount),
        nonce: BigInt(nonce),
      },
      fee: 5000, // 5000 lamports fee
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    // Sign transaction (user provides signing function)
    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    // Send to blockchain (real RPC call)
    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send a stake delegation transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.staker - Staker address (base58)
   * @param {string} params.validator - Validator address (base58)
   * @param {number} params.amount - Amount to stake in lamports
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async stake({ staker, validator, amount, signFn }) {
    if (!staker || !validator || !amount === undefined) {
      throw new AetherSDKError('staker, validator, and amount are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: staker,
      tx_type: 'Stake',
      payload: {
        validator: validator,
        amount: BigInt(amount),
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send an unstake (withdraw) transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.stakeAccount - Stake account address (base58)
   * @param {number} params.amount - Amount to unstake in lamports
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async unstake({ stakeAccount, amount, signFn }) {
    if (!stakeAccount || !amount === undefined) {
      throw new AetherSDKError('stakeAccount and amount are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: stakeAccount,
      tx_type: 'Unstake',
      payload: {
        stake_account: stakeAccount,
        amount: BigInt(amount),
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send a claim rewards transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.stakeAccount - Stake account address (base58)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async claimRewards({ stakeAccount, signFn }) {
    if (!stakeAccount) {
      throw new AetherSDKError('stakeAccount is required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: stakeAccount,
      tx_type: 'ClaimRewards',
      payload: {
        stake_account: stakeAccount,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  // ============================================================
  // NFT Methods - Real blockchain calls for NFT operations
  // ============================================================

  /**
   * Create a new NFT
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.creator - Creator address (base58)
   * @param {string} params.metadataUrl - URL to NFT metadata (JSON)
   * @param {number} params.royalties - Royalty basis points (e.g., 500 = 5%)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt with NFT ID
   */
  async createNFT({ creator, metadataUrl, royalties, signFn }) {
    if (!creator || !metadataUrl || royalties === undefined) {
      throw new AetherSDKError('creator, metadataUrl, and royalties are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: creator,
      tx_type: 'CreateNFT',
      payload: {
        metadata_url: metadataUrl,
        royalties: royalties,
      },
      fee: 10000, // Higher fee for NFT creation
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Transfer an NFT to another address
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.from - Current owner address (base58)
   * @param {string} params.nftId - NFT ID
   * @param {string} params.to - Recipient address (base58)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async transferNFT({ from, nftId, to, signFn }) {
    if (!from || !nftId || !to) {
      throw new AetherSDKError('from, nftId, and to are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: from,
      tx_type: 'TransferNFT',
      payload: {
        nft_id: nftId,
        recipient: to,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Update NFT metadata URL
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.creator - NFT creator/owner address (base58)
   * @param {string} params.nftId - NFT ID
   * @param {string} params.metadataUrl - New metadata URL
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async updateMetadata({ creator, nftId, metadataUrl, signFn }) {
    if (!creator || !nftId || !metadataUrl) {
      throw new AetherSDKError('creator, nftId, and metadataUrl are required', 'VALIDATION_ERROR');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new AetherSDKError('signFn is required', 'VALIDATION_ERROR');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: creator,
      tx_type: 'UpdateMetadata',
      payload: {
        nft_id: nftId,
        metadata_url: metadataUrl,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  // ============================================================
  // NFT Query Methods - Real blockchain calls for NFT data
  // ============================================================

  /**
   * Get NFT details by ID
   * RPC: GET /v1/nft/<id>
   * 
   * @param {string} nftId - NFT ID
   * @returns {Promise<Object>} NFT details: { id, creator, metadata_url, royalties, supply, max_supply, created_at, update_authority }
   */
  async getNFT(nftId) {
    if (!nftId) throw new AetherSDKError('NFT ID is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/nft/${nftId}`);
        return result;
      },
      'getNFT'
    );
  }

  /**
   * Get NFT holdings for an address
   * RPC: GET /v1/nft-holdings/<address>
   * 
   * @param {string} address - Account address
   * @returns {Promise<Array>} List of NFT holdings with { id, amount, metadata_url }
   */
  async getNFTHoldings(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/nft-holdings/${address}`);
        return result.holdings ?? result.nfts ?? result ?? [];
      },
      'getNFTHoldings'
    );
  }

  /**
   * Get all NFTs created by an address
   * RPC: GET /v1/nft-created/<address>
   * 
   * @param {string} address - Creator address
   * @returns {Promise<Array>} List of created NFTs
   */
  async getNFTsByCreator(address) {
    if (!address) throw new AetherSDKError('Address is required', 'VALIDATION_ERROR');
    return this._executeWithRetry(
      async () => {
        const result = await this._httpGet(`/v1/nft-created/${address}`);
        return result.nfts ?? result.created ?? result ?? [];
      },
      'getNFTsByCreator'
    );
  }

  // ============================================================
  // Utilities
  // ============================================================

  /**
   * Get client statistics
   * @returns {Object} Request statistics
   */
  getStats() {
    return {
      ...this.stats,
      circuitBreaker: this.circuitBreaker.getState(),
      rateLimiter: {
        rps: this.rateLimiter.rps,
        burst: this.rateLimiter.burst,
        tokens: this.rateLimiter.tokens,
      },
    };
  }

  /**
   * Reset circuit breaker
   */
  resetCircuitBreaker() {
    this.circuitBreaker = new CircuitBreaker(
      this.circuitBreaker.threshold,
      this.circuitBreaker.resetTimeoutMs
    );
  }

  /**
   * Close the client and cleanup resources
   */
  destroy() {
    this.rateLimiter.destroy();
  }
}

// ============================================================
// Convenience Functions (for quick one-off calls)
// ============================================================

/**
 * Create a new AetherClient instance
 * @param {Object} options - Client options
 * @returns {AetherClient}
 */
function createClient(options = {}) {
  return new AetherClient(options);
}

/**
 * Quick slot check (uses default RPC)
 * @returns {Promise<number>} Current slot
 */
async function getSlot() {
  const client = new AetherClient();
  try {
    return await client.getSlot();
  } finally {
    client.destroy();
  }
}

/**
 * Quick balance check (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<number>} Balance in lamports
 */
async function getBalance(address) {
  const client = new AetherClient();
  try {
    return await client.getBalance(address);
  } finally {
    client.destroy();
  }
}

/**
 * Quick health check (uses default RPC)
 * @returns {Promise<string>} 'ok' if healthy
 */
async function getHealth() {
  const client = new AetherClient();
  try {
    return await client.getHealth();
  } finally {
    client.destroy();
  }
}

/**
 * Get current block height (uses default RPC)
 * @returns {Promise<number>} Block height
 */
async function getBlockHeight() {
  const client = new AetherClient();
  try {
    return await client.getBlockHeight();
  } finally {
    client.destroy();
  }
}

/**
 * Get epoch info (uses default RPC)
 * @returns {Promise<Object>} Epoch info
 */
async function getEpoch() {
  const client = new AetherClient();
  try {
    return await client.getEpochInfo();
  } finally {
    client.destroy();
  }
}

/**
 * Get TPS (uses default RPC)
 * @returns {Promise<number>} Transactions per second
 */
async function getTPS() {
  const client = new AetherClient();
  try {
    return await client.getTPS();
  } finally {
    client.destroy();
  }
}

/**
 * Get supply info (uses default RPC)
 * @returns {Promise<Object>} Supply info
 */
async function getSupply() {
  const client = new AetherClient();
  try {
    return await client.getSupply();
  } finally {
    client.destroy();
  }
}

/**
 * Get fees info (uses default RPC)
 * @returns {Promise<Object>} Fee info
 */
async function getFees() {
  const client = new AetherClient();
  try {
    return await client.getFees();
  } finally {
    client.destroy();
  }
}

/**
 * Get validators list (uses default RPC)
 * @returns {Promise<Array>} List of validators
 */
async function getValidators() {
  const client = new AetherClient();
  try {
    return await client.getValidators();
  } finally {
    client.destroy();
  }
}

/**
 * Get peers list (uses default RPC)
 * @returns {Promise<Array>} List of peers
 */
async function getPeers() {
  const client = new AetherClient();
  try {
    return await client.getClusterPeers();
  } finally {
    client.destroy();
  }
}

/**
 * Get slot production stats (uses default RPC)
 * @returns {Promise<Object>} Slot production stats
 */
async function getSlotProduction() {
  const client = new AetherClient();
  try {
    return await client.getSlotProduction();
  } finally {
    client.destroy();
  }
}

/**
 * Get account info (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Object>} Account info
 */
async function getAccount(address) {
  const client = new AetherClient();
  try {
    return await client.getAccount(address);
  } finally {
    client.destroy();
  }
}

/**
 * Get stake positions (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Stake positions
 */
async function getStakePositions(address) {
  const client = new AetherClient();
  try {
    return await client.getStakePositions(address);
  } finally {
    client.destroy();
  }
}

/**
 * Get rewards info (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Object>} Rewards info
 */
async function getRewards(address) {
  const client = new AetherClient();
  try {
    return await client.getRewards(address);
  } finally {
    client.destroy();
  }
}

/**
 * Get validator APY (uses default RPC)
 * @param {string} validatorAddr - Validator address
 * @returns {Promise<Object>} APY info
 */
async function getValidatorAPY(validatorAddr) {
  const client = new AetherClient();
  try {
    return await client.getValidatorAPY(validatorAddr);
  } finally {
    client.destroy();
  }
}

/**
 * Get transaction by signature (uses default RPC)
 * @param {string} signature - Transaction signature
 * @returns {Promise<Object>} Transaction info
 */
async function getTransaction(signature) {
  const client = new AetherClient();
  try {
    return await client.getTransaction(signature);
  } finally {
    client.destroy();
  }
}

/**
 * Get recent transactions (uses default RPC)
 * @param {string} address - Account address
 * @param {number} limit - Max transactions
 * @returns {Promise<Array>} Recent transactions
 */
async function getRecentTransactions(address, limit = 20) {
  const client = new AetherClient();
  try {
    return await client.getRecentTransactions(address, limit);
  } finally {
    client.destroy();
  }
}

/**
 * Get transaction history with signatures for an address (uses default RPC)
 * @param {string} address - Account address
 * @param {number} limit - Max transactions
 * @returns {Promise<Object>} Transaction history with signatures and details
 */
async function getTransactionHistory(address, limit = 20) {
  const client = new AetherClient();
  try {
    return await client.getTransactionHistory(address, limit);
  } finally {
    client.destroy();
  }
}

/**
 * Get all SPL token accounts for a wallet (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Token accounts with mint, amount, decimals
 */
async function getTokenAccounts(address) {
  const client = new AetherClient();
  try {
    return await client.getTokenAccounts(address);
  } finally {
    client.destroy();
  }
}

/**
 * Get all stake accounts for a wallet (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Stake accounts list
 */
async function getStakeAccounts(address) {
  const client = new AetherClient();
  try {
    return await client.getStakeAccounts(address);
  } finally {
    client.destroy();
  }
}

/**
 * Send transaction (uses default RPC)
 * @param {Object} tx - Signed transaction
 * @returns {Promise<Object>} Transaction receipt
 */
async function sendTransaction(tx) {
  const client = new AetherClient();
  try {
    return await client.sendTransaction(tx);
  } finally {
    client.destroy();
  }
}

/**
 * Ping RPC endpoint
 * @param {string} rpcUrl - RPC URL to ping
 * @returns {Promise<Object>} Ping result with latency
 */
async function ping(rpcUrl) {
  const client = new AetherClient({ rpcUrl });
  const start = Date.now();
  try {
    await client.getSlot();
    return { ok: true, latency: Date.now() - start, rpc: rpcUrl || DEFAULT_RPC_URL };
  } catch (err) {
    return { ok: false, error: err.message, rpc: rpcUrl || DEFAULT_RPC_URL };
  } finally {
    client.destroy();
  }
}

// ============================================================
// NFT Convenience Functions (for quick one-off calls)
// ============================================================

/**
 * Get NFT details by ID (uses default RPC)
 * @param {string} nftId - NFT ID
 * @returns {Promise<Object>} NFT details
 */
async function getNFT(nftId) {
  const client = new AetherClient();
  try {
    return await client.getNFT(nftId);
  } finally {
    client.destroy();
  }
}

/**
 * Get NFT holdings for an address (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} List of NFT holdings
 */
async function getNFTHoldings(address) {
  const client = new AetherClient();
  try {
    return await client.getNFTHoldings(address);
  } finally {
    client.destroy();
  }
}

/**
 * Get NFTs created by an address (uses default RPC)
 * @param {string} address - Creator address
 * @returns {Promise<Array>} List of created NFTs
 */
async function getNFTsByCreator(address) {
  const client = new AetherClient();
  try {
    return await client.getNFTsByCreator(address);
  } finally {
    client.destroy();
  }
}

// ============================================================
// Exports
// ============================================================

module.exports = {
  // Main class
  AetherClient,
  
  // Error classes
  AetherSDKError,
  NetworkTimeoutError,
  RPCError,
  RateLimitError,
  CircuitBreakerOpenError,
  
  // Factory function
  createClient,
  
  // Convenience functions (all chain queries)
  getSlot,
  getBlockHeight,
  getEpoch,
  getAccount,
  getBalance,
  getTransaction,
  getRecentTransactions,
  getTransactionHistory,
  getTokenAccounts,
  getStakeAccounts,
  getValidators,
  getTPS,
  getSupply,
  getSlotProduction,
  getFees,
  getStakePositions,
  getRewards,
  getValidatorAPY,
  getPeers,
  getHealth,
  
  // NFT queries
  getNFT,
  getNFTHoldings,
  getNFTsByCreator,
  
  // Transactions
  sendTransaction,
  
  // Utilities
  ping,
  
  // Low-level RPC
  rpcGet,
  rpcPost,
  
  // Constants
  DEFAULT_RPC_URL,
  DEFAULT_TIMEOUT_MS,
  DEFAULT_RETRY_ATTEMPTS,
  DEFAULT_RETRY_DELAY_MS,
  DEFAULT_BACKOFF_MULTIPLIER,
  DEFAULT_MAX_RETRY_DELAY_MS,
  DEFAULT_RATE_LIMIT_RPS,
  DEFAULT_RATE_LIMIT_BURST,
  DEFAULT_CIRCUIT_BREAKER_THRESHOLD,
  DEFAULT_CIRCUIT_BREAKER_RESET_MS,
};
