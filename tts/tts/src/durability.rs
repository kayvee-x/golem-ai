use crate::config;
use std::time::Duration;

/// Gets the configured number of max retries for TTS calls
pub fn max_retries() -> u32 {
    config::get_max_retries()
}

/// Gets the configured timeout duration for TTS requests
pub fn request_timeout() -> Duration {
    Duration::from_secs(config::get_timeout_seconds())
}

/// Calculates backoff delay for retries using exponential strategy
pub fn backoff_delay(retry_count: u32) -> Duration {
    let base = 2u64.pow(retry_count.min(5)); // Cap at 2^5 = 32s
    Duration::from_secs(base)
}