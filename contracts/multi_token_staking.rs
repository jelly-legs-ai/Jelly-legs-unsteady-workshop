use anchor_lang::prelude::*;
use std::collections::HashMap;

declare_id!("StakingPool1111111111111111111111111111111");

#[program]
pub mod multi_token_staking {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        reward_tokens: Vec<Pubkey>,
        reward_rates: Vec<u64>,
        lockup_periods: Vec<u64>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        pool.authority = ctx.accounts.authority.key();
        pool.reward_tokens = reward_tokens;
        pool.reward_rates = reward_rates;
        pool.lockup_periods = lockup_periods;
        pool.total_staked = 0;
        pool.active_stakers = 0;
        pool.last_update = Clock::get()?.unix_timestamp;
        pool.bump = ctx.bumps.staking_pool;
        
        Ok(())
    }

    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        duration_weeks: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        let user = &mut ctx.accounts.user_account;
        let clock = Clock::get()?;
        
        require!(amount > 0, StakingError::InvalidAmount);
        require!(
            duration_weeks >= 1 && duration_weeks <= 52,
            StakingError::InvalidDuration
        );
        
        let lockup_end = clock.unix_timestamp + (duration_weeks as i64 * 7 * 24 * 60 * 60);
        
        let stake_entry = StakeEntry {
            owner: ctx.accounts.user.key(),
            amount,
            lockup_end,
            reward_tokens_earned: HashMap::new(),
            last_claim: clock.unix_timestamp,
            multiplier: calculate_multiplier(duration_weeks),
        };
        
        user.stakes.push(stake_entry);
        pool.total_staked += amount;
        pool.active_stakers += 1;
        
        // Transfer tokens to staking account
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;
        
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>, stake_index: usize) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        let pool = &mut ctx.accounts.staking_pool;
        let clock = Clock::get()?;
        
        require!(stake_index < user.stakes.len(), StakingError::InvalidStakeIndex);
        
        let stake = &mut user.stakes[stake_index];
        let time_elapsed = (clock.unix_timestamp - stake.last_claim) as u64;
        
        // Calculate rewards for each token
        let mut total_rewards: Vec<(Pubkey, u64)> = Vec::new();
        for (i, token_key) in pool.reward_tokens.iter().enumerate() {
            let rate = pool.reward_rates[i];
            let multiplier = stake.multiplier;
            let reward = (time_elapsed as u64)
                .checked_mul(rate)
                .and_then(|r| r.checked_mul(stake.amount))
                .and_then(|r| r.checked_div(10000))
                .and_then(|r| r.checked_mul(multiplier))
                .unwrap_or(0);
            
            if reward > 0 {
                total_rewards.push((*token_key, reward));
            }
        }
        
        stake.last_claim = clock.unix_timestamp;
        pool.last_update = clock.unix_timestamp;
        
        // Emit claim event
        emit!(RewardClaimed {
            user: ctx.accounts.user.key(),
            stake_index,
            rewards: total_rewards.clone(),
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, stake_index: usize) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        let pool = &mut ctx.accounts.staking_pool;
        let clock = Clock::get()?;
        
        require!(stake_index < user.stakes.len(), StakingError::InvalidStakeIndex);
        
        let stake = &user.stakes[stake_index];
        require!(
            clock.unix_timestamp >= stake.lockup_end,
            StakingError::LockupNotEnded
        );
        
        let amount = stake.amount;
        pool.total_staked -= amount;
        pool.active_stakers -= 1;
        
        // Remove stake
        user.stakes.remove(stake_index);
        
        // Transfer tokens back
        let seeds = &[
            b"staking_pool",
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.staking_vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.staking_pool.to_account_info(),
            },
            signer,
        );
        transfer(cpi_ctx, amount)?;
        
        emit!(Unstaked {
            user: ctx.accounts.user.key(),
            amount,
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    pub fn update_pool_params(
        ctx: Context<UpdatePool>,
        new_rates: Option<Vec<u64>>,
        new_lockups: Option<Vec<u64>>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        
        require!(
            ctx.accounts.authority.key() == pool.authority,
            StakingError::Unauthorized
        );
        
        if let Some(rates) = new_rates {
            pool.reward_rates = rates;
        }
        if let Some(lockups) = new_lockups {
            pool.lockup_periods = lockups;
        }
        
        pool.last_update = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<StakingPool>()
    )]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<UserStakeAccount>()
    )]
    pub user_account: Account<'info, UserStakeAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub user_account: Account<'info, UserStakeAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub user_account: Account<'info, UserStakeAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_vault: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    pub authority: Signer<'info>,
}

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub reward_tokens: Vec<Pubkey>,
    pub reward_rates: Vec<u64>,
    pub lockup_periods: Vec<u64>,
    pub total_staked: u64,
    pub active_stakers: u64,
    pub last_update: i64,
    pub bump: u8,
}

#[account]
pub struct UserStakeAccount {
    pub owner: Pubkey,
    pub stakes: Vec<StakeEntry>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StakeEntry {
    pub owner: Pubkey,
    pub amount: u64,
    pub lockup_end: i64,
    pub reward_tokens_earned: HashMap<Pubkey, u64>,
    pub last_claim: i64,
    pub multiplier: u64,
}

#[error_code]
pub enum StakingError {
    #[msg("Invalid staking amount")]
    InvalidAmount,
    #[msg("Invalid lockup duration")]
    InvalidDuration,
    #[msg("Lockup period has not ended")]
    LockupNotEnded,
    #[msg("Invalid stake index")]
    InvalidStakeIndex,
    #[msg("Unauthorized access")]
    Unauthorized,
}

fn calculate_multiplier(duration_weeks: u64) -> u64 {
    match duration_weeks {
        1..=4 => 100,    // 1x
        5..=12 => 120,   // 1.2x
        13..=26 => 150, // 1.5x
        27..=52 => 200, // 2x
        _ => 100,
    }
}

#[event]
pub struct RewardClaimed {
    pub user: Pubkey,
    pub stake_index: usize,
    pub rewards: Vec<(Pubkey, u64)>,
    pub timestamp: i64,
}

#[event]
pub struct Unstaked {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
