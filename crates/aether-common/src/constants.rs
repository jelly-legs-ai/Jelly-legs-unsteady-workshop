//! AETHER Protocol Constants

/// Target slot time in milliseconds (maintained from Solana)
pub const SLOT_TIME_MS: u64 = 400;

/// Slots per epoch (~2 days at 400ms per slot)
pub const SLOTS_PER_EPOCH: u64 = 432_000;

/// Minimum stake for validator (10,000 AETH)
pub const MINIMUM_STAKE_AETH: u64 = 10_000_000_000_000; // 10,000 * 10^9 (9 decimals)

/// Minimum stake for AI validator Tier 1 (100,000 AETH)
pub const MINIMUM_AI_STAKE_AETH: u64 = 100_000_000_000_000; // 100,000 * 10^9

/// Warmup period for stake activation (2 epochs)
pub const STAKE_WARMUP_EPOCHS: u64 = 2;

/// Cooldown period for stake deactivation (4 epochs)
pub const STAKE_COOLDOWN_EPOCHS: u64 = 4;

/// Maximum validator commission (100%)
pub const MAX_VALIDATOR_COMMISSION: u8 = 100;

/// Default validator commission (10%)
pub const DEFAULT_VALIDATOR_COMMISSION: u8 = 10;

/// Total AETH supply (1 billion)
pub const TOTAL_SUPPLY_AETH: u64 = 1_000_000_000_000_000_000; // 1B * 10^9

/// Base transaction fee (in lamports)
pub const BASE_TRANSACTION_FEE: u64 = 5_000;

/// AI verification fee (in lamports)
pub const AI_VERIFICATION_FEE: u64 = 1_000_000; // 0.001 AETH

/// Critical lane multiplier for priority fees
pub const CRITICAL_LANE_MULTIPLIER: u64 = 10;

/// High lane multiplier for priority fees
pub const HIGH_LANE_MULTIPLIER: u64 = 5;

/// Burn percentage of transaction fees (50%)
pub const TRANSACTION_FEE_BURN_PERCENT: u8 = 50;

/// Burn percentage of priority fees (100%)
pub const PRIORITY_FEE_BURN_PERCENT: u8 = 100;

/// Target TPS for deflationary trigger (40,000)
pub const DEFLATIONARY_TPS_THRESHOLD: u64 = 40_000;

/// AI oracle base voting power
pub const AI_ORACLE_BASE_VOTING_POWER: u64 = 100_000_000;

/// Number of AI oracles in the council (5-9)
pub const AI_ORACLE_COUNT: usize = 7;

/// Guardian threshold for bridge (13-of-19)
pub const GUARDIAN_THRESHOLD: u8 = 13;

/// Total guardians in bridge
pub const TOTAL_GUARDIANS: u8 = 19;

/// Daily bridge transfer limit (10M AETH)
pub const BRIDGE_DAILY_LIMIT: u64 = 10_000_000_000_000_000;

/// Maximum single bridge transfer (1M AETH)
pub const BRIDGE_MAX_SINGLE_TRANSFER: u64 = 1_000_000_000_000_000;

/// Stake lock period in epochs (~2 days per epoch)
pub const STAKE_LOCK_EPOCHS: u64 = 2;
