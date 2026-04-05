// Utility helpers for AeTHer Chain
// Common helper functions used across contracts

/// Calculate percentage of a value
pub fn percentage(value: u64, percent: u64) -> u64 {
    (value * percent) / 100
}

/// Calculate weighted average
pub fn weighted_average(values: &[u64], weights: &[u64]) -> u64 {
    if values.is_empty() || weights.is_empty() || values.len() != weights.len() {
        return 0;
    }
    let total_weight: u64 = weights.iter().sum();
    if total_weight == 0 {
        return 0;
    }
    let weighted_sum: u64 = values.iter().zip(weights.iter()).map(|(v, w)| v * w).sum();
    weighted_sum / total_weight
}

/// Clamp a value between min and max
pub fn clamp(value: u64, min: u64, max: u64) -> u64 {
    if value < min { min } else if value > max { max } else { value }
}

/// Linear interpolation between two values
pub fn lerp(start: u64, end: u64, t: u64) -> u64 {
    // t is 0-100 percentage
    start + ((end - start) * t) / 100
}

/// Calculate compound interest
pub fn compound_interest(principal: u64, rate: u64, periods: u64) -> u64 {
    // rate is in basis points (e.g., 500 = 5%)
    if periods == 0 {
        return principal;
    }
    let mut result = principal;
    for _ in 0..periods {
        result = result + (result * rate) / 10000;
    }
    result
}

/// Unix timestamp helper
pub fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Validate address format (basic check)
pub fn is_valid_address(addr: &str) -> bool {
    addr.starts_with("0x") && addr.len() == 42
}

/// Format large numbers with K/M/B suffixes
pub fn format_number(num: u64) -> String {
    if num >= 1_000_000_000 {
        format!("{:.1}B", num as f64 / 1_000_000_000.0)
    } else if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

/// Calculate APY (Annual Percentage Yield) from base rate
pub fn calculate_apy(base_rate: u64, compounding_periods: u64) -> u64 {
    // Returns APY in basis points
    if compounding_periods == 0 {
        return base_rate;
    }
    let mut result: u64 = 10000; // 100% in basis points
    for _ in 0..compounding_periods {
        result = result + (result * base_rate) / 10000;
    }
    result - 10000
}

/// Calculate reward multiplier based on stake duration
pub fn stake_duration_multiplier(duration_epochs: u64) -> f64 {
    match duration_epochs {
        0..=30 => 1.0,      // < 1 month: 1x
        31..=90 => 1.25,    // 1-3 months: 1.25x
        91..=180 => 1.5,   // 3-6 months: 1.5x
        181..=365 => 2.0,  // 6-12 months: 2x
        _ => 3.0,          // > 1 year: 3x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(1000, 10), 100);
        assert_eq!(percentage(1000, 50), 500);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(0, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_weighted_average() {
        let values = [10, 20, 30];
        let weights = [1, 1, 1];
        assert_eq!(weighted_average(&values, &weights), 20);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_stake_duration_multiplier() {
        assert_eq!(stake_duration_multiplier(30), 1.0);
        assert_eq!(stake_duration_multiplier(60), 1.25);
        assert_eq!(stake_duration_multiplier(200), 2.0);
    }
}
