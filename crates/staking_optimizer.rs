// AeTHer Chain - Staking Portfolio Optimizer
// Automatically optimizes staking positions across tiers and validators

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub validator_id: String,
    pub amount: u64,
    pub tier: StakingTier,
    pub lock_duration: u64,
    pub rewards_accrued: u64,
    pub last_claim: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StakingTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl StakingTier {
    pub fn min_stake(&self) -> u64 {
        match self {
            StakingTier::Bronze => 100,
            StakingTier::Silver => 1_000,
            StakingTier::Gold => 10_000,
            StakingTier::Platinum => 100_000,
            StakingTier::Diamond => 1_000_000,
        }
    }

    pub fn reward_multiplier(&self) -> f64 {
        match self {
            StakingTier::Bronze => 1.0,
            StakingTier::Silver => 1.25,
            StakingTier::Gold => 1.5,
            StakingTier::Platinum => 2.0,
            StakingTier::Diamond => 3.0,
        }
    }

    pub fn apy(&self) -> f64 {
        match self {
            StakingTier::Bronze => 0.05,
            StakingTier::Silver => 0.075,
            StakingTier::Gold => 0.10,
            StakingTier::Platinum => 0.15,
            StakingTier::Diamond => 0.20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub id: String,
    pub name: String,
    pub uptime: f64,
    pub commission: f64,
    pub delegators: u64,
    pub total_stake: u64,
    pub avg_reward_rate: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub current_positions: Vec<StakingPosition>,
    pub suggested_actions: Vec<StakingAction>,
    pub projected_annual_yield: u64,
    pub risk_score: f64,
    pub diversification_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakingAction {
    Stake { validator: String, amount: u64, tier: StakingTier },
    Unstake { validator: String, amount: u64 },
    Restake { from_validator: String, to_validator: String, amount: u64 },
    UpgradeTier { validator: String, new_tier: StakingTier },
    ClaimRewards { validator: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalancingPlan {
    pub actions: Vec<StakingAction>,
    pub total_reallocation: u64,
    pub expected_apy_improvement: f64,
    pub risk_reduction: f64,
}

pub struct StakingOptimizer {
    validators: HashMap<String, ValidatorInfo>,
    positions: HashMap<String, Vec<StakingPosition>>,
    risk_tolerance: f64,
}

impl StakingOptimizer {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            positions: HashMap::new(),
            risk_tolerance: 0.5,
        }
    }

    pub fn with_risk_tolerance(mut self, tolerance: f64) -> Self {
        self.risk_tolerance = tolerance.clamp(0.0, 1.0);
        self
    }

    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.validators.insert(validator.id.clone(), validator);
    }

    pub fn add_position(&mut self, wallet: &str, position: StakingPosition) {
        self.positions
            .entry(wallet.to_string())
            .or_default()
            .push(position);
    }

    pub fn analyze_portfolio(&self, wallet: &str) -> OptimizationResult {
        let positions = self.positions.get(wallet).cloned().unwrap_or_default();
        
        let mut projected_annual_yield = 0u64;
        let mut total_stake = 0u64;
        let mut tier_exposure: HashMap<StakingTier, u64> = HashMap::new();
        let mut validator_exposure: HashMap<String, u64> = HashMap::new();

        for pos in &positions {
            let validator = self.validators.get(&pos.validator_id);
            let effective_apy = pos.tier.apy() * pos.tier.reward_multiplier();
            let base_yield = (pos.amount as f64 * effective_apy) as u64;
            let commission_deduction = validator.map(|v| v.commission).unwrap_or(0.05);
            
            projected_annual_yield += (base_yield as f64 * (1.0 - commission_deduction)) as u64;
            total_stake += pos.amount;

            *tier_exposure.entry(pos.tier).or_default() += pos.amount;
            *validator_exposure.entry(pos.validator_id.clone()).or_default() += pos.amount;
        }

        let diversification_score = self.calculate_diversification_score(&positions);
        let risk_score = self.calculate_risk_score(&positions, total_stake);

        let suggested_actions = self.generate_suggestions(wallet, &positions, total_stake);

        OptimizationResult {
            current_positions: positions,
            suggested_actions,
            projected_annual_yield,
            risk_score,
            diversification_score,
        }
    }

    fn calculate_diversification_score(&self, positions: &[StakingPosition]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let total: u64 = positions.iter().map(|p| p.amount).sum();
        if total == 0 {
            return 0.0;
        }

        // Calculate Herfindahl-Hirschman Index (HHI)
        let mut hhi = 0.0;
        for pos in positions {
            let share = pos.amount as f64 / total as f64;
            hhi += share * share;
        }

        // Convert HHI to diversification score (lower HHI = higher diversification)
        (1.0 - hhi) * 100.0
    }

    fn calculate_risk_score(&self, positions: &[StakingPosition], total_stake: u64) -> f64 {
        if positions.is_empty() || total_stake == 0 {
            return 1.0;
        }

        let mut risk_factors = Vec::new();

        // Concentration risk
        for pos in positions {
            let concentration = pos.amount as f64 / total_stake as f64;
            if concentration > 0.3 {
                risk_factors.push((concentration - 0.3) * 2.0);
            }
        }

        // Validator concentration risk
        let mut validator_stake: HashMap<String, u64> = HashMap::new();
        for pos in positions {
            *validator_stake.entry(pos.validator_id.clone()).or_default() += pos.amount;
        }

        for (_, stake) in &validator_stake {
            let concentration = *stake as f64 / total_stake as f64;
            if concentration > 0.25 {
                risk_factors.push((concentration - 0.25) * 3.0);
            }
        }

        // Tier diversification
        let tier_count = positions.iter().map(|p| p.tier).collect::<std::collections::HashSet<_>>().len();
        if tier_count < 3 {
            risk_factors.push((3 - tier_count) as f64 * 0.15);
        }

        let total_risk: f64 = risk_factors.iter().sum();
        total_risk.min(1.0)
    }

    fn generate_suggestions(
        &self,
        wallet: &str,
        positions: &[StakingPosition],
        total_stake: u64,
    ) -> Vec<StakingAction> {
        let mut actions = Vec::new();
        let positions = positions.to_vec();

        // Check for high-concentration positions
        for pos in &positions {
            if total_stake > 0 {
                let concentration = pos.amount as f64 / total_stake as f64;
                if concentration > 0.4 {
                    // Suggest spreading to lower-tier positions
                    let excess = ((concentration - 0.25) * total_stake as f64) as u64;
                    if excess > pos.tier.min_stake() {
                        // Find validators in a different tier
                        if let Some(better_validator) = self.find_validator_for_tier(&StakingTier::Silver, pos.amount / 2) {
                            actions.push(StakingAction::Restake {
                                from_validator: pos.validator_id.clone(),
                                to_validator: better_validator,
                                amount: excess / 2,
                            });
                        }
                    }
                }
            }
        }

        // Check for upgrade opportunities
        for pos in &positions {
            if pos.amount > pos.tier.min_stake() * 10 && pos.tier != StakingTier::Diamond {
                let upgrade_tier = match pos.tier {
                    StakingTier::Bronze => Some(StakingTier::Silver),
                    StakingTier::Silver => Some(StakingTier::Gold),
                    StakingTier::Gold => Some(StakingTier::Platinum),
                    StakingTier::Platinum => Some(StakingTier::Diamond),
                    StakingTier::Diamond => None,
                };

                if let Some(new_tier) = upgrade_tier {
                    actions.push(StakingAction::UpgradeTier {
                        validator: pos.validator_id.clone(),
                        new_tier,
                    });
                }
            }
        }

        // Check for unclaimed rewards
        for pos in &positions {
            if pos.rewards_accrued > pos.amount / 100 {
                actions.push(StakingAction::ClaimRewards {
                    validator: pos.validator_id.clone(),
                });
            }
        }

        actions
    }

    fn find_validator_for_tier(&self, tier: &StakingTier, _amount: u64) -> Option<String> {
        self.validators
            .values()
            .filter(|v| v.is_active && v.uptime > 0.95)
            .max_by(|a, b| {
                let score_a = a.avg_reward_rate * (1.0 - a.commission) * a.uptime;
                let score_b = b.avg_reward_rate * (1.0 - b.commission) * b.uptime;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|v| v.id.clone())
    }

    pub fn create_rebalancing_plan(&self, wallet: &str) -> Option<RebalancingPlan> {
        let optimization = self.analyze_portfolio(wallet);

        if optimization.suggested_actions.is_empty() {
            return None;
        }

        let total_reallocation: u64 = optimization
            .suggested_actions
            .iter()
            .filter_map(|action| match action {
                StakingAction::Stake { amount, .. } => Some(*amount),
                StakingAction::Unstake { amount, .. } => Some(*amount),
                StakingAction::Restake { amount, .. } => Some(*amount),
                _ => None,
            })
            .sum();

        // Calculate expected improvements
        let current_apy = if optimization.projected_annual_yield > 0 {
            optimization.projected_annual_yield as f64
        } else {
            0.05
        };

        let expected_apy_improvement = 0.02; // Simplified estimate
        let risk_reduction = (100.0 - optimization.risk_score * 100.0) * 0.1;

        Some(RebalancingPlan {
            actions: optimization.suggested_actions,
            total_reallocation,
            expected_apy_improvement,
            risk_reduction,
        })
    }

    pub fn get_health_check(&self, wallet: &str) -> PortfolioHealth {
        let positions = self.positions.get(wallet).cloned().unwrap_or_default();
        
        let issues = Vec::new();
        let total_stake: u64 = positions.iter().map(|p| p.amount).sum();

        // Check for issues
        if positions.is_empty() {
            return PortfolioHealth {
                score: 0.0,
                status: HealthStatus::Critical,
                issues: vec!["No staking positions found".to_string()],
                recommendations: vec!["Start staking to earn rewards".to_string()],
            };
        }

        // Single validator risk
        let validator_count: std::collections::HashSet<_> = positions.iter().map(|p| &p.validator_id).collect();
        if validator_count.len() == 1 {
            issues.push("All stake concentrated with single validator".to_string());
        }

        // Tier concentration
        let tier_count: std::collections::HashSet<_> = positions.iter().map(|p| p.tier).collect();
        if tier_count.len() < 2 {
            issues.push("Limited tier diversification".to_string());
        }

        // High-concentration positions
        for pos in &positions {
            if total_stake > 0 {
                let conc = pos.amount as f64 / total_stake as f64;
                if conc > 0.5 {
                    issues.push(format!(
                        "Position {} has {:.0}% of portfolio",
                        pos.validator_id.chars().take(8).collect::<String>(),
                        conc * 100.0
                    ));
                }
            }
        }

        let score = ((100.0 - issues.len() as f64 * 10.0) - self.calculate_risk_score(&positions, total_stake) * 20.0)
            .clamp(0.0, 100.0);

        let status = if score >= 80.0 {
            HealthStatus::Healthy
        } else if score >= 50.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };

        let recommendations = vec![
            "Consider spreading stake across multiple validators".to_string(),
            "Diversify across different staking tiers".to_string(),
            "Claim and restake rewards regularly".to_string(),
        ];

        PortfolioHealth {
            score,
            status,
            issues,
            recommendations,
        }
    }

    /// Calculate compound growth projection over time
    pub fn compound_projection(
        &self,
        wallet: &str,
        months: u64,
        monthly_contribution: u64,
    ) -> CompoundProjection {
        let positions = self.positions.get(wallet).cloned().unwrap_or_default();
        
        let monthly_rate = positions.iter()
            .map(|p| {
                let effective_apy = p.tier.apy() * p.tier.reward_multiplier();
                let validator = self.validators.get(&p.validator_id);
                let commission = validator.map(|v| v.commission).unwrap_or(0.05);
                effective_apy * (1.0 - commission) / 12.0
            })
            .sum::<f64>() / positions.len().max(1) as f64;

        let current_stake: u64 = positions.iter().map(|p| p.amount).sum();
        
        let mut balance = current_stake as f64;
        let mut projections = Vec::new();
        
        for month in 0..=months {
            projections.push(ProjectionPoint {
                month,
                balance: balance as u64,
                rewards_accrued: (balance - current_stake as f64).max(0.0) as u64,
            });
            balance = balance * (1.0 + monthly_rate) + monthly_contribution as f64;
        }

        CompoundProjection {
            monthly_rate,
            current_stake,
            final_balance: balance as u64,
            total_rewards: (balance - current_stake as f64 - (monthly_contribution as f64 * months as f64)).max(0.0) as u64,
            monthly_contribution,
            projections,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundProjection {
    pub monthly_rate: f64,
    pub current_stake: u64,
    pub final_balance: u64,
    pub total_rewards: u64,
    pub monthly_contribution: u64,
    pub projections: Vec<ProjectionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionPoint {
    pub month: u64,
    pub balance: u64,
    pub rewards_accrued: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealth {
    pub score: f64,
    pub status: HealthStatus,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Warning => write!(f, "Warning"),
            HealthStatus::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_multipliers() {
        assert_eq!(StakingTier::Bronze.reward_multiplier(), 1.0);
        assert_eq!(StakingTier::Silver.reward_multiplier(), 1.25);
        assert_eq!(StakingTier::Gold.reward_multiplier(), 1.5);
        assert_eq!(StakingTier::Diamond.reward_multiplier(), 3.0);
    }

    #[test]
    fn test_diversification_score() {
        let optimizer = StakingOptimizer::new();
        let positions = vec![
            StakingPosition {
                validator_id: "v1".to_string(),
                amount: 500,
                tier: StakingTier::Bronze,
                lock_duration: 90,
                rewards_accrued: 25,
                last_claim: 0,
            },
            StakingPosition {
                validator_id: "v2".to_string(),
                amount: 500,
                tier: StakingTier::Silver,
                lock_duration: 180,
                rewards_accrued: 37,
                last_claim: 0,
            },
        ];

        let score = optimizer.calculate_diversification_score(&positions);
        assert!(score > 50.0);
    }

    #[test]
    fn test_risk_score() {
        let optimizer = StakingOptimizer::new();
        let positions = vec![
            StakingPosition {
                validator_id: "v1".to_string(),
                amount: 900,
                tier: StakingTier::Gold,
                lock_duration: 180,
                rewards_accrued: 45,
                last_claim: 0,
            },
        ];

        let score = optimizer.calculate_risk_score(&positions, 1000);
        assert!(score > 0.5); // High concentration = high risk
    }
}

// ============================================================================
// Additional Utility Functions for Production Use
// ============================================================================

impl StakingOptimizer {
    /// Calculate compound growth projection over time periods
    pub fn project_compound_growth(
        &self,
        principal: u64,
        apy: f64,
        periods: u32,
        compounds_per_year: u32,
    ) -> CompoundProjection {
        let rate_per_period = apy / compounds_per_year as f64;
        let periods_f = compounds_per_year as f64 * periods as f64;
        
        // A = P(1 + r/n)^(nt)
        let growth_factor = (1.0 + rate_per_period).powf(periods_f);
        let final_amount = (principal as f64 * growth_factor) as u64;
        let total_earned = final_amount.saturating_sub(principal);

        CompoundProjection {
            principal,
            final_amount,
            total_earned,
            growth_factor,
            periods,
            apy,
            compounds_per_year,
        }
    }

    /// Calculate time to reach a target amount with compound interest
    pub fn time_to_target(&self, principal: u64, target: u64, apy: f64) -> Option<u32> {
        if principal >= target || apy <= 0.0 {
            return None;
        }
        
        let rate = apy / 365.0; // Daily compounding
        let target_ratio = target as f64 / principal as f64;
        
        // ln(target/principal) / ln(1 + rate)
        let days = (target_ratio.ln() / (1.0 + rate).ln()).ceil() as u32;
        
        Some(days.max(1))
    }

    /// Calculate optimal staking split across tiers for a given amount
    pub fn calculate_optimal_split(&self, total_amount: u64) -> Vec<(StakingTier, u64)> {
        let mut remaining = total_amount;
        let mut splits = Vec::new();
        
        // Priority: Diamond -> Platinum -> Gold -> Silver -> Bronze
        let tiers = [
            StakingTier::Diamond,
            StakingTier::Platinum,
            StakingTier::Gold,
            StakingTier::Silver,
            StakingTier::Bronze,
        ];
        
        for tier in tiers {
            let min = tier.min_stake();
            if remaining >= min {
                // Allocate minimum for this tier, rest continues
                splits.push((tier, min));
                remaining = remaining.saturating_sub(min);
            }
        }
        
        // If we have remaining after minimum allocations, put in highest tier
        if remaining > 0 {
            if let Some((last_tier, _)) = splits.last_mut() {
                *last_tier = StakingTier::Diamond;
            }
            if let Some((tier, amount)) = splits.last_mut() {
                *amount += remaining;
            }
        }
        
        splits
    }

    /// Get recommended validators for a given tier
    pub fn get_recommended_validators(&self, tier: &StakingTier, limit: usize) -> Vec<ValidatorInfo> {
        self.validators
            .values()
            .filter(|v| {
                v.is_active && 
                v.uptime > 0.98 && 
                v.total_stake >= tier.min_stake()
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

/// Compound growth projection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundProjection {
    pub principal: u64,
    pub final_amount: u64,
    pub total_earned: u64,
    pub growth_factor: f64,
    pub periods: u32,
    pub apy: f64,
    pub compounds_per_year: u32,
}

impl CompoundProjection {
    /// Get monthly breakdown for year 1
    pub fn monthly_breakdown_year1(&self) -> Vec<MonthlySnapshot> {
        let monthly_rate = self.apy / 12.0;
        let mut snapshots = Vec::new();
        let mut current = self.principal as f64;
        
        for month in 1..=12 {
            current *= 1.0 + monthly_rate;
            snapshots.push(MonthlySnapshot {
                month,
                amount: current as u64,
                earned: current as u64 - self.principal,
            });
        }
        
        snapshots
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySnapshot {
    pub month: u32,
    pub amount: u64,
    pub earned: u64,
}
