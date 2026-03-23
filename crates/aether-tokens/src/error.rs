//! Token contract errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Insufficient balance for transfer")]
    InsufficientBalance,
    
    #[error("Insufficient allowance for delegated transfer")]
    InsufficientAllowance,
    
    #[error("Operation would exceed maximum supply")]
    ExceedsMaxSupply,
    
    #[error("Staking amount below minimum required")]
    BelowMinStake,
    
    #[error("Insufficient staking rewards")]
    InsufficientRewards,
    
    #[error("Invalid token mint")]
    InvalidMint,
    
    #[error("Arithmetic overflow")]
    Overflow,
    
    #[error("Unauthorized operation")]
    Unauthorized,
    
    #[error("Token is frozen")]
    TokenFrozen,
    
    #[error("Invalid instruction data")]
    InvalidInstruction,
}
