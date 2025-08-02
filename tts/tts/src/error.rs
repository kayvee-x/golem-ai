use crate::golem::tts::tts::TtsError;
use reqwest::StatusCode;
use std::error::Error;

/// Error for unsupported operations/features
pub fn unsupported(what: impl AsRef<str>) -> TtsError {
    TtsError::UnsupportedFeature(format!("Unsupported: {}", what.as_ref()))
}

/// Wraps a `reqwest::Error` into a backend error with context
pub fn from_reqwest_error(context: impl AsRef<str>, err: reqwest::Error) -> TtsError {
    TtsError::BackendError(format!("{}: {}", context.as_ref(), err))
}

/// Wraps any generic error into a backend error with context
pub fn from_generic_error<T: Error>(context: impl AsRef<str>, err: T) -> TtsError {
    TtsError::BackendError(format!("{}: {}", context.as_ref(), err))
}

/// Converts an HTTP status code + optional body into a TTS error
pub fn error_from_status(status: StatusCode, body: Option<String>) -> TtsError {
    match status {
        StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = body
                .and_then(|b| b.parse::<u32>().ok())
                .unwrap_or(60); // fallback to 60s retry
            TtsError::RateLimited(retry_after)
        }
        StatusCode::UNAUTHORIZED
        | StatusCode::FORBIDDEN
        | StatusCode::PAYMENT_REQUIRED => {
            TtsError::BackendError("Authentication failed".into())
        }
        s if s.is_client_error() => TtsError::InvalidQuery,
        _ => {
            let message = match body {
                Some(b) => format!("HTTP {}: {}", status, b),
                None => format!("HTTP {}", status),
            };
            TtsError::BackendError(message)
        }
    }
}