use crate::TtsError;
use reqwest::StatusCode;

/// Converts an HTTP status code + optional body into a TTS error
pub fn error_from_status(status: StatusCode, body: Option<String>) -> TtsError {
    match status {
        StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = body.and_then(|b| b.parse::<u32>().ok()).unwrap_or(60); // fallback to 60s retry
            TtsError::RateLimited(retry_after)
        }
        StatusCode::UNAUTHORIZED => {
            TtsError::Unauthorized(body.unwrap_or_else(|| "Unauthorized".to_string()))
        }
        StatusCode::FORBIDDEN | StatusCode::PAYMENT_REQUIRED => {
            TtsError::AccessDenied(body.unwrap_or_else(|| "Access Denied".to_string()))
        }
        _ => (if status.is_client_error() {
            TtsError::NetworkError
        } else {
            TtsError::InternalError
        })(format_status(status, body.as_deref())),
    }
}

fn format_status(status: StatusCode, body: Option<&str>) -> String {
    let mut out = format!("HTTP {status}");
    if let Some(b) = body {
        out.reserve(b.len() + 2);
        out.push_str(": ");
        out.push_str(b);
    }
    out
}
