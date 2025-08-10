use crate::TtsError;
use std::env;
use std::ffi::OsStr;

/// Creates a standardized error when a config key is missing
fn missing_key_error(key: &str) -> TtsError {
    TtsError::InvalidConfiguration(format!("Missing configuration key: {key}"))
}

/// Gets a required config value, or returns an error via a fail/succeed flow
pub fn with_config_key<R>(
    key: impl AsRef<OsStr>,
    fail: impl FnOnce(TtsError) -> R,
    succeed: impl FnOnce(String) -> R,
) -> R {
    let key_str = key.as_ref().to_string_lossy().to_string();
    match env::var(&key) {
        Ok(value) => succeed(value),
        Err(_) => fail(missing_key_error(&key_str)),
    }
}

/// Gets a required config value, returns a Result
pub fn validate_config_key(key: impl AsRef<OsStr>) -> Result<String, TtsError> {
    let key_str = key.as_ref().to_string_lossy().to_string();
    env::var(&key).map_err(|_| missing_key_error(&key_str))
}

/// Gets an optional config value, returns None if missing
pub fn get_optional_config(key: impl AsRef<OsStr>) -> Option<String> {
    env::var(key).ok()
}

/// Gets a config value or falls back to a provided default
pub fn get_config_with_default(key: impl AsRef<OsStr>, default: impl Into<String>) -> String {
    env::var(key).unwrap_or_else(|_| default.into())
}

/// Gets multiple required config values, or errors if any is missing
pub fn get_required_config_keys(keys: &[&str]) -> Result<Vec<String>, TtsError> {
    let mut values = Vec::new();
    for key in keys {
        match env::var(key) {
            Ok(value) => values.push(value),
            Err(_) => return Err(missing_key_error(key)),
        }
    }
    Ok(values)
}

/// Gets multiple optional config values, missing ones return empty strings
pub fn get_optional_config_keys(keys: &[&str]) -> Vec<String> {
    keys.iter()
        .map(|key| env::var(key).unwrap_or_default())
        .collect()
}

/// Number of retries for failed TTS calls
pub fn get_max_retries() -> u32 {
    get_config_with_default("TTS_MAX_RETRIES", "3")
        .parse()
        .unwrap_or(3)
}

/// Request timeout in seconds
pub fn get_timeout_seconds() -> u64 {
    get_config_with_default("TTS_TIMEOUT_SECONDS", "30")
        .parse()
        .unwrap_or(30)
}

/// Default voice to use for TTS (optional override)
pub fn get_default_voice() -> String {
    get_config_with_default("TTS_DEFAULT_VOICE", "default")
}

/// Whether to log generated audio URLs (e.g., for debugging)
pub fn get_log_audio_urls() -> bool {
    get_config_with_default("TTS_LOG_AUDIO_URLS", "false")
        .parse()
        .unwrap_or(false)
}
