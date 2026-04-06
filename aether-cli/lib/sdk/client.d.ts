/**
 * AetherChain SDK - TypeScript Definitions
 * Real blockchain RPC client for the Aether testnet
 */

export interface SlotInfo {
  slot: number;
  block_hash: string;
  parent_block_hash: string;
  healthy: boolean;
  error: string | null;
}

export interface BlockHeightInfo {
  blockHeight: number;
  slot: number;
}

export interface BlockInfo {
  slot: number;
  timestamp: number;
  block_hash: string;
  previous_block_hash: string;
  poh_seed: string;
  transaction_count: number;
}

export interface GenesisInfo {
  chain_id: string;
  genesis_hash: string;
}

export interface EpochInfo {
  epoch: number;
  slot_index: number;
  slots_in_epoch: number;
  absolute_slot: number;
  transaction_count: number;
}

export interface BlockProductionStats {
  blocks_produced: number;
  entries_produced: number;
  epoch: number;
}

export interface Validator {
  identity_pubkey: string;
  activated_stake: number;
  commission: number;
  active: boolean;
}

export interface ValidatorInfo {
  tier: string;
  consensus_weight: number;
  can_produce_blocks: boolean;
  can_vote: boolean;
}

export interface VoteAccounts {
  vote_accounts: any[];
  total_stake: number;
}

export interface AccountInfo {
  address: string;
  lamports: number;
  owner: string;
  data_size: number;
  rent_epoch: number;
}

export interface TotalSupply {
  total_supply: string;
  unit: string;
}

export interface TransactionRequest {
  tx_type: string;
  signer: string;
  signature: string;
  payload?: object;
  fee?: number;
}

export interface TransactionResponse {
  signature: string;
  slot: number;
}

export interface TransactionError {
  error: string;
}

export interface TransactionStatus {
  signature: string;
  slot: number;
  block_hash: string;
  success: boolean;
  error: string | null;
  timestamp: number;
}

export interface HealthStatus {
  status: string;
}

export interface PingResult {
  reachable: boolean;
  slot: number | null;
  healthy: boolean;
  blockHash?: string;
  error?: string;
}

export declare class AetherClient {
  rpcUrl: string;

  constructor(rpcUrl?: string);

  /**
   * GET /v1/slot — Current slot info
   */
  getSlot(): Promise<SlotInfo>;

  /**
   * GET /v1/blockheight — Current block height (alias for slot)
   */
  getBlockHeight(): Promise<BlockHeightInfo>;

  /**
   * GET /v1/block?slot=N — Get block by slot number
   */
  getBlock(slot: number): Promise<BlockInfo | null>;

  /**
   * GET /v1/genesis — Genesis configuration
   */
  getGenesis(): Promise<GenesisInfo>;

  /**
   * GET /v1/epoch — Epoch information
   */
  getEpoch(): Promise<EpochInfo>;

  /**
   * GET /v1/block_production — Block production stats
   */
  getBlockProduction(): Promise<BlockProductionStats>;

  /**
   * GET /v1/validators — Connected validators
   */
  getValidators(): Promise<Validator[]>;

  /**
   * GET /v1/validator/info — Current validator tier
   */
  getValidatorInfo(): Promise<ValidatorInfo>;

  /**
   * GET /v1/voteAccounts — Vote accounts
   */
  getVoteAccounts(): Promise<VoteAccounts>;

  /**
   * GET /v1/account/<address> — Get account info
   */
  getAccount(address: string): Promise<AccountInfo>;

  /**
   * GET /v1/total_supply — Total token supply
   */
  getTotalSupply(): Promise<TotalSupply>;

  /**
   * POST /v1/tx — Submit a transaction
   */
  sendTransaction(tx: TransactionRequest): Promise<TransactionResponse | TransactionError>;

  /**
   * GET /v1/tx/<signature> — Get transaction status
   */
  getTransaction(signature: string): Promise<TransactionStatus | null>;

  /**
   * GET /health — Health check
   */
  health(): Promise<HealthStatus>;

  /**
   * Check if chain is reachable and healthy
   */
  ping(): Promise<PingResult>;
}

export { AetherClient as default };
