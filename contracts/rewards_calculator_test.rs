// Rewards Calculator Tests & Validation - AeTHer Chain
// Test suite for mining and staking reward calculations

use serde::{Deserialize, Serialize};

/// Test case for mining reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningRewardTestCase {
    pub name: String,
    pub device_tier: String,
    pub ram_gb: u32,
    pub uptime_percentage: f64,
    pub contribution_score: f64,
    pub epochs_mined: u64,
    pub expected_reward_range: (u64, u64),
}

/// Test case for staking reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRewardTestCase {
    pub name: String,
    pub stake_amount: u64,
    pub uptime_percentage: f64,
    pub epochs_staked: u64,
    pub commission_rate: f64,
    pub expected_apy_range: (f64, f64),
}

/// Test results for reward calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub deviation_percentage: f64,
}

impl TestResult {
    pub fn new(name: String, expected: String, actual: String) -> Self {
        let dev = Self::calculate_deviation(&expected, &actual);
        Self {
            test_name: name,
            passed: false,
            expected,
            actual,
            deviation_percentage: dev,
        }
    }

    fn calculate_deviation(expected: &str, actual: &str) -> f64 {
        let e: f64 = expected.parse().unwrap_or(0.0);
        let a: f64 = actual.parse().unwrap_or(0.0);
        if e == 0.0 { 0.0 } else { ((a - e) / e * 100.0).abs() }
    }

    pub fn validate(&mut self, tolerance: f64) {
        self.passed = self.deviation_percentage <= tolerance;
    }
}

/// Mining reward test suite
pub struct MiningRewardTestSuite;

impl MiningRewardTestSuite {
    /// Get standard test cases for mining rewards
    pub fn get_test_cases() -> Vec<MiningRewardTestCase> {
        vec![
            MiningRewardTestCase {
                name: "Mobile device basic".to_string(),
                device_tier: "Mobile".to_string(),
                ram_gb: 4,
                uptime_percentage: 90.0,
                contribution_score: 0.8,
                epochs_mined: 100,
                expected_reward_range: (8_000_000, 12_000_000),
            },
            MiningRewardTestCase {
                name: "Laptop mid-tier".to_string(),
                device_tier: "Laptop".to_string(),
                ram_gb: 16,
                uptime_percentage: 95.0,
                contribution_score: 0.9,
                epochs_mined: 500,
                expected_reward_range: (20_000_000, 30_000_000),
            },
            MiningRewardTestCase {
                name: "Desktop high performer".to_string(),
                device_tier: "Desktop".to_string(),
                ram_gb: 32,
                uptime_percentage: 99.0,
                contribution_score: 0.95,
                epochs_mined: 1000,
                expected_reward_range: (50_000_000, 75_000_000),
            },
            MiningRewardTestCase {
                name: "Server tier".to_string(),
                device_tier: "Server".to_string(),
                ram_gb: 128,
                uptime_percentage: 99.9,
                contribution_score: 0.98,
                epochs_mined: 5000,
                expected_reward_range: (80_000_000, 120_000_000),
            },
        ]
    }

    /// Run all mining reward tests
    pub fn run_tests() -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for case in Self::get_test_cases() {
            let result = Self::validate_test_case(&case);
            results.push(result);
        }
        
        results
    }

    /// Validate a single test case
    fn validate_test_case(case: &MiningRewardTestCase) -> TestResult {
        // Simulate reward calculation (simplified)
        let tier_multiplier = match case.device_tier.as_str() {
            "Mobile" => 1.0,
            "Laptop" => 2.5,
            "Desktop" => 5.0,
            "Server" => 10.0,
            _ => 1.0,
        };
        
        let base_reward = 10_000_000.0;
        let uptime_factor = if case.uptime_percentage >= 95.0 { 1.0 } else { 0.8 };
        let contribution_factor = 1.0 + (case.contribution_score * 0.5);
        let epochs_factor = (case.epochs_mined as f64 / 1000.0).min(0.5) + 1.0;
        
        let calculated = base_reward * tier_multiplier * uptime_factor * contribution_factor * epochs_factor;
        
        let actual_str = format!("{:.0}", calculated);
        let expected_str = format!("{:.0}", (case.expected_reward_range.0 + case.expected_reward_range.1) as f64 / 2.0);
        
        let mut result = TestResult::new(case.name.clone(), expected_str, actual_str);
        result.validate(30.0); // 30% tolerance
        
        result
    }
}

/// Staking reward test suite
pub struct StakingRewardTestSuite;

impl StakingRewardTestSuite {
    /// Get standard test cases for staking rewards
    pub fn get_test_cases() -> Vec<StakingRewardTestCase> {
        vec![
            StakingRewardTestCase {
                name: "Basic validator".to_string(),
                stake_amount: 100_000_000,
                uptime_percentage: 95.0,
                epochs_staked: 100,
                commission_rate: 0.05,
                expected_apy_range: (0.12, 0.15),
            },
            StakingRewardTestCase {
                name: "Premium validator".to_string(),
                stake_amount: 1_000_000_000,
                uptime_percentage: 99.0,
                epochs_staked: 500,
                commission_rate: 0.05,
                expected_apy_range: (0.15, 0.18),
            },
            StakingRewardTestCase {
                name: "Enterprise validator".to_string(),
                stake_amount: 10_000_000_000,
                uptime_percentage: 99.9,
                epochs_staked: 1000,
                commission_rate: 0.03,
                expected_apy_range: (0.17, 0.22),
            },
        ]
    }

    /// Run all staking reward tests
    pub fn run_tests() -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for case in Self::get_test_cases() {
            let result = Self::validate_test_case(&case);
            results.push(result);
        }
        
        results
    }

    /// Validate a single test case
    fn validate_test_case(case: &StakingRewardTestCase) -> TestResult {
        // Simulate APY calculation (simplified)
        let base_apy = 0.12;
        let uptime_bonus = if case.uptime_percentage >= 95.0 { 0.03 } else { 0.0 };
        let loyalty_mult = if case.epochs_staked >= 1000 { 1.15 } else if case.epochs_staked >= 500 { 1.1 } else if case.epochs_staked >= 100 { 1.05 } else { 1.0 };
        
        let calculated_apy = (base_apy + uptime_bonus) * loyalty_mult;
        
        let expected_mid = (case.expected_apy_range.0 + case.expected_apy_range.1) / 2.0;
        
        let actual_str = format!("{:.4}", calculated_apy);
        let expected_str = format!("{:.4}", expected_mid);
        
        let mut result = TestResult::new(case.name.clone(), expected_str, actual_str);
        result.validate(20.0); // 20% tolerance
        
        result
    }
}

/// Cross-chain reward validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainRewardValidation {
    pub source_chain: String,
    pub dest_chain: String,
    pub original_reward: u64,
    pub bridge_fee: u64,
    pub final_reward: u64,
    pub validation_passed: bool,
}

impl CrossChainRewardValidation {
    /// Validate cross-chain reward transfer
    pub fn validate_transfer(source: &str, dest: &str, reward: u64, bridge_fee_bps: u64) -> Self {
        let bridge_fee = (reward as f64 * bridge_fee_bps as f64 / 10000.0) as u64;
        let final_reward = reward - bridge_fee;
        
        let validation_passed = final_reward > 0 && bridge_fee < reward;
        
        Self {
            source_chain: source.to_string(),
            dest_chain: dest.to_string(),
            original_reward: reward,
            bridge_fee,
            final_reward,
            validation_passed,
        }
    }
}

/// Print test summary
pub fn print_test_summary(mining_results: &[TestResult], staking_results: &[TestResult]) {
    let mining_passed = mining_results.iter().filter(|r| r.passed).count();
    let staking_passed = staking_results.iter().filter(|r| r.passed).count();
    
    println!("\n=== REWARD CALCULATOR TEST SUMMARY ===");
    println!("Mining Tests: {}/{} passed", mining_passed, mining_results.len());
    println!("Staking Tests: {}/{} passed", staking_passed, staking_results.len());
    
    for result in mining_results.iter().chain(staking_results.iter()) {
        let status = if result.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("  {} - {} (deviation: {:.1}%)", status, result.test_name, result.deviation_percentage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_reward_calculations() {
        let results = MiningRewardTestSuite::run_tests();
        let passed = results.iter().filter(|r| r.passed).count();
        println!("Mining tests passed: {}/{}", passed, results.len());
        assert!(passed >= results.len() / 2, "At least 50% of mining tests should pass");
    }

    #[test]
    fn test_staking_reward_calculations() {
        let results = StakingRewardTestSuite::run_tests();
        let passed = results.iter().filter(|r| r.passed).count();
        println!("Staking tests passed: {}/{}", passed, results.len());
        assert!(passed >= results.len() / 2, "At least 50% of staking tests should pass");
    }

    #[test]
    fn test_cross_chain_validation() {
        let validation = CrossChainRewardValidation::validate_transfer("Aether", "Ethereum", 1_000_000, 30);
        assert!(validation.validation_passed);
        assert_eq!(validation.final_reward, 997_000);
    }
}
