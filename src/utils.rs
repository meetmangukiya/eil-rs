use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in seconds
pub fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Convert fee percentage (0.0 to 1.0) to numerator out of 10_000
pub fn fee_percent_to_numerator(percent: f64) -> alloy::primitives::U256 {
    alloy::primitives::U256::from((percent * 10_000.0) as u64)
}
