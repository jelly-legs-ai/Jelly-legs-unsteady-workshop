/**
 * @jellylegsai/aether-sdk
 * 
 * Official Aether Blockchain SDK TypeScript Definitions
 * Real HTTP RPC calls to Aether nodes with full type safety
 */

// ============================================================================
// Error Types
// ============================================================================

export class AetherSDKError extends Error {
  name: 'AetherSDKError';
  code: string;
  details: Record<string, unknown>;
  timestamp: string;
}

export class NetworkTimeoutError extends AetherSDKError {
  name: 'NetworkTimeoutError';
  code: 'NETWORK_TIMEOUT';
}

export class RPCError extends AetherSDKError {
  name: 'RPCError';
  code: 'RPC_ERROR';
}

export class RateLimitError extends AetherSDKError {
  name: 'RateLimitError';
  code: 'RATE_LIMIT';
}

export class CircuitBreakerOpenError extends AetherSDKError {
  name: 'CircuitBreakerOpenError';
  code: 'CIRCUIT_BREAKER_OPEN';
}

// ============================================================================
// Configuration Types
// ============================================================================

export interface AetherClientOptions {
  /** RPC endpoint URL (default: http://127.0.0.1:8899) */
  rpcUrl?: string;
  /** Request timeout in milliseconds (default: 10000) */
  timeoutMs?: number;
  /** Number of retry attempts (default: 3) */
  retryAttempts?: number;
  /** Initial retry delay in milliseconds (default: 1000) */
  retryDelayMs?: number;
  /** Backoff multiplier for retries (default: 2) */
  backoffMultiplier?: number;
  /** Maximum retry delay in milliseconds (default: 30000) */
  maxRetryDelayMs?: number;
  /** Rate limit requests per second (default: 10) */
  rateLimitRps?: number;
  /** Rate limit burst capacity (default: 20) */
  rateLimitBurst?: number;
  /** Circuit breaker failure threshold (default: 5) */
  circuitBreakerThreshold?: number;
  /** Circuit breaker reset timeout in milliseconds (default: 60000) */
  circuitBreakerResetMs?: number;
}

export interface CircuitBreakerState {
  state: 'CLOSED' | 'OPEN' | 'HALF_OPEN';
  failureCount: number;
  nextAttempt: number | null;
}

export interface RateLimiterState {
  rps: number;
  burst: number;
  tokens: number;
}

export interface ClientStats {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  retriedRequests: number;
  rateLimitedRequests: number;
  circuitBreakerBlocked: number;
  circuitBreaker: CircuitBreakerState;
  rateLimiter: RateLimiterState;
}

// ============================================================================
// Blockchain Types
// ============================================================================

export interface AccountInfo {
  lamports: number;
  owner: string;
  data: unknown;
  executable: boolean;
  rent_epoch: number;
}

export interface EpochInfo {
  epoch: number;
  slotIndex: number;
  slotsInEpoch: number;
  absoluteSlot: number;
  block_height?: number;
  epoch_schedule?: {
    first_normal_epoch: number;
    first_normal_slot: number;
    leader_schedule_slot_offset: number;
    slots_per_epoch: number;
    warmup: boolean;
  };
}

export interface SupplyInfo {
  total: number;
  circulating: number;
  nonCirculating: number;
  total_staked?: number;
}

export interface FeeInfo {
  lamportsPerSignature: number;
  feeCalculator?: {
    lamportsPerSignature: number;
  };
}

export interface BlockhashInfo {
  blockhash: string;
  lastValidBlockHeight: number;
}

export interface ValidatorInfo {
  address?: string;
  vote_account?: string;
  identity?: string;
  node_pubkey?: string;
  pubkey?: string;
  stake_lamports?: number;
  activated_stake?: number;
  stake?: number;
  lamports?: number;
  commission?: number;
  commission_bps?: number;
  apy?: number;
  return_rate?: number;
  name?: string;
  moniker?: string;
  tier?: 'full' | 'lite' | 'observer';
  active?: boolean;
  delinquent?: boolean;
  version?: string;
  agent?: string;
  app_version?: string;
  ip?: string;
  remote?: string;
  last_vote?: number;
  lastVote?: number;
  epoch?: number;
  uptime?: number;
  score?: number;
}

export interface TransactionInfo {
  signature: string;
  slot: number;
  timestamp: number;
  tx_type?: string;
  signer?: string;
  fee?: number;
  payload?: {
    type?: string;
    data?: {
      recipient?: string;
      amount?: string | number | bigint;
      validator?: string;
      stake_account?: string;
      nonce?: number;
    };
  };
  confirmations?: number;
  status?: 'confirmed' | 'finalized' | 'failed' | 'pending';
}

export interface TransactionReceipt {
  signature: string;
  txid?: string;
  slot: number;
  confirmed: boolean;
  block_height?: number;
  error?: string;
}

export interface StakePosition {
  pubkey?: string;
  publicKey?: string;
  account?: string;
  address?: string;
  validator?: string;
  delegate?: string;
  voter?: string;
  lamports?: number;
  stake_lamports?: number;
  activation_epoch?: number;
  deactivation_epoch?: number;
  status?: string;
  state?: string;
  stake_type?: string;
  type?: string;
  pending_rewards?: number;
  rewards?: number;
}

export interface RewardsInfo {
  total: number;
  pending: number;
  pending_rewards?: number;
  amount?: number;
  validator?: string;
  rewards_per_epoch?: string;
  total_network_stake?: string;
}

export interface PeerInfo {
  address?: string;
  pubkey?: string;
  id?: string;
  tier?: string;
  node_type?: string;
  score?: number;
  uptime?: number;
  uptime_seconds?: number;
}

export interface SlotProductionStats {
  slotsProduced?: number;
  byIdentity?: Record<string, number[]>;
  samplePeriodSecs?: number;
}

export interface TokenAccount {
  mint: string;
  amount: number;
  decimals: number;
  owner?: string;
}

export interface StakeAccount {
  pubkey?: string;
  publicKey?: string;
  account?: string;
  lamports?: number;
  stake_lamports?: number;
  validator?: string;
  voter?: string;
  status?: string;
  state?: string;
}

// ============================================================================
// NFT Types
// ============================================================================

export interface NFTInfo {
  id: string;
  creator?: string;
  mint_authority?: string;
  metadata_url?: string;
  metadata?: string;
  royalties?: number;
  royalty_bps?: number;
  supply?: number;
  current_supply?: number;
  max_supply?: number;
  created_at?: number;
  update_authority?: string;
}

export interface NFTHolding {
  nft_id?: string;
  id?: string;
  mint?: string;
  amount?: number;
  balance?: number;
  acquired_at?: number;
  metadata_url?: string;
  metadata?: string;
}

// ============================================================================
// Transaction Builder Types
// ============================================================================

export interface TransferParams {
  from: string;
  to: string;
  amount: number;
  nonce: number;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface StakeParams {
  staker: string;
  validator: string;
  amount: number;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface UnstakeParams {
  stakeAccount: string;
  amount: number;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface ClaimRewardsParams {
  stakeAccount: string;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface NFTCreateParams {
  creator: string;
  metadataUrl: string;
  royalties: number;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface NFTTransferParams {
  from: string;
  nftId: string;
  to: string;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface NFTUpdateMetadataParams {
  creator: string;
  nftId: string;
  metadataUrl: string;
  signFn: (tx: TransactionData, blockhash: string) => Promise<string> | string;
}

export interface TransactionData {
  signature: string;
  signer: string;
  tx_type: string;
  payload: {
    type?: string;
    data?: Record<string, unknown>;
  };
  fee: number;
  slot: number;
  timestamp: number;
}

// ============================================================================
// Ping Types
// ============================================================================

export interface PingResult {
  ok: boolean;
  latency: number;
  rpc: string;
  error?: string;
}

// ============================================================================
// AetherClient Class
// ============================================================================

export class AetherClient {
  readonly rpcUrl: string;
  readonly timeoutMs: number;
  readonly retryAttempts: number;
  readonly retryDelayMs: number;
  readonly backoffMultiplier: number;
  readonly maxRetryDelayMs: number;

  constructor(options?: AetherClientOptions);

  // Core RPC Methods
  getSlot(): Promise<number>;
  getBlockHeight(): Promise<number>;
  getAccountInfo(address: string): Promise<AccountInfo>;
  getAccount(address: string): Promise<AccountInfo>;
  getBalance(address: string): Promise<number>;
  getEpochInfo(): Promise<EpochInfo>;
  getTransaction(signature: string): Promise<TransactionInfo>;
  sendTransaction(tx: TransactionData): Promise<TransactionReceipt>;
  getRecentBlockhash(): Promise<BlockhashInfo>;
  getClusterPeers(): Promise<PeerInfo[]>;
  getValidators(): Promise<ValidatorInfo[]>;
  getSupply(): Promise<SupplyInfo>;
  getHealth(): Promise<string>;
  getVersion(): Promise<{ aetherCore?: string; featureSet?: string }>;
  getTPS(): Promise<number | null>;
  getFees(): Promise<FeeInfo>;
  getSlotProduction(): Promise<SlotProductionStats>;

  // Stake Operations
  getStakePositions(address: string): Promise<StakePosition[]>;
  getRewards(address: string): Promise<RewardsInfo>;
  getStakeAccounts(address: string): Promise<StakeAccount[]>;

  // Transaction Queries
  getRecentTransactions(address: string, limit?: number): Promise<TransactionInfo[]>;
  getTransactionHistory(address: string, limit?: number): Promise<{
    signatures: string[];
    transactions: TransactionInfo[];
    address: string;
  }>;

  // Token Operations
  getTokenAccounts(address: string): Promise<TokenAccount[]>;

  // NFT Operations
  createNFT(params: NFTCreateParams): Promise<TransactionReceipt>;
  transferNFT(params: NFTTransferParams): Promise<TransactionReceipt>;
  updateMetadata(params: NFTUpdateMetadataParams): Promise<TransactionReceipt>;
  getNFT(nftId: string): Promise<NFTInfo>;
  getNFTHoldings(address: string): Promise<NFTHolding[]>;
  getNFTsByCreator(address: string): Promise<NFTInfo[]>;

  // Transaction Helpers
  transfer(params: TransferParams): Promise<TransactionReceipt>;
  stake(params: StakeParams): Promise<TransactionReceipt>;
  unstake(params: UnstakeParams): Promise<TransactionReceipt>;
  claimRewards(params: ClaimRewardsParams): Promise<TransactionReceipt>;

  // Utilities
  getStats(): ClientStats;
  resetCircuitBreaker(): void;
  destroy(): void;
}

// ============================================================================
// Token Bucket Rate Limiter
// ============================================================================

export class TokenBucketRateLimiter {
  readonly rps: number;
  readonly burst: number;
  tokens: number;
  lastRefill: number;

  constructor(rps?: number, burst?: number);
  refill(): void;
  processQueue(): void;
  acquire(tokens?: number): Promise<void>;
  destroy(): void;
}

// ============================================================================
// Circuit Breaker
// ============================================================================

export class CircuitBreaker {
  readonly threshold: number;
  readonly resetTimeoutMs: number;
  failureCount: number;
  state: 'CLOSED' | 'OPEN' | 'HALF_OPEN';
  nextAttempt: number;

  constructor(threshold?: number, resetTimeoutMs?: number);
  canExecute(): boolean;
  recordSuccess(): void;
  recordFailure(): void;
  getState(): CircuitBreakerState;
}

// ============================================================================
// Low-level RPC
// ============================================================================

export function rpcGet(
  rpcUrl: string,
  path: string,
  timeout?: number,
  retries?: number
): Promise<unknown>;

export function rpcPost(
  rpcUrl: string,
  path: string,
  body: Record<string, unknown>,
  timeout?: number,
  retries?: number
): Promise<unknown>;

// ============================================================================
// Convenience Functions
// ============================================================================

export function createClient(options?: AetherClientOptions): AetherClient;

// One-off queries (create client, call method, destroy)
export function getSlot(): Promise<number>;
export function getBlockHeight(): Promise<number>;
export function getEpoch(): Promise<EpochInfo>;
export function getAccount(address: string): Promise<AccountInfo>;
export function getBalance(address: string): Promise<number>;
export function getTransaction(signature: string): Promise<TransactionInfo>;
export function getRecentTransactions(address: string, limit?: number): Promise<TransactionInfo[]>;
export function getTransactionHistory(address: string, limit?: number): Promise<{
  signatures: string[];
  transactions: TransactionInfo[];
  address: string;
}>;
export function getTokenAccounts(address: string): Promise<TokenAccount[]>;
export function getStakeAccounts(address: string): Promise<StakeAccount[]>;
export function getValidators(): Promise<ValidatorInfo[]>;
export function getTPS(): Promise<number | null>;
export function getSupply(): Promise<SupplyInfo>;
export function getSlotProduction(): Promise<SlotProductionStats>;
export function getFees(): Promise<FeeInfo>;
export function getStakePositions(address: string): Promise<StakePosition[]>;
export function getRewards(address: string): Promise<RewardsInfo>;
export function getValidatorAPY(validatorAddr: string): Promise<{ apy?: number; error?: string }>;
export function getPeers(): Promise<PeerInfo[]>;
export function getHealth(): Promise<string>;

// NFT queries
export function getNFT(nftId: string): Promise<NFTInfo>;
export function getNFTHoldings(address: string): Promise<NFTHolding[]>;
export function getNFTsByCreator(address: string): Promise<NFTInfo[]>;

// Transaction submission
export function sendTransaction(tx: TransactionData): Promise<TransactionReceipt>;

// Utilities
export function ping(rpcUrl?: string): Promise<PingResult>;

// ============================================================================
// Constants
// ============================================================================

export const DEFAULT_RPC_URL: string;
export const DEFAULT_TIMEOUT_MS: number;
export const DEFAULT_RETRY_ATTEMPTS: number;
export const DEFAULT_RETRY_DELAY_MS: number;
export const DEFAULT_BACKOFF_MULTIPLIER: number;
export const DEFAULT_MAX_RETRY_DELAY_MS: number;
export const DEFAULT_RATE_LIMIT_RPS: number;
export const DEFAULT_RATE_LIMIT_BURST: number;
export const DEFAULT_CIRCUIT_BREAKER_THRESHOLD: number;
export const DEFAULT_CIRCUIT_BREAKER_RESET_MS: number;
