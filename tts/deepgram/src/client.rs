use golem_tts::config::with_config_key;
use golem_tts::golem::tts::types::TtsError;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[allow(dead_code)]
struct ErrorResponse {
    err_code: String,
    err_msg: String,
    request_id: String,
}

pub struct Client {
    client: reqwest::Client,
    base_url: String,
}
impl Client {
    pub fn new() -> Result<Self, TtsError> {
        use reqwest::header::*;
        let api_key = with_config_key("DEEPGRAM_API_KEY", Err, Ok)?;
        let base_url = with_config_key(
            "DEEPGRAM_BASE_URL",
            |_| "https://api.deepgram.com".to_string(),
            |x| x,
        );
        if api_key.len() != 40 || !api_key.as_bytes().iter().all(u8::is_ascii_hexdigit) {
            return Err(TtsError::InvalidConfiguration(
                "Deepgram API keys should be 40 hex characters".to_string(),
            ));
        }
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Token {api_key}"))
                .map_err(|e| TtsError::InvalidConfiguration(format!("Invalid API key: {e}")))?,
        );
        Ok(Self {
            client: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .map_err(|e| {
                    TtsError::InvalidConfiguration(format!("Failed to build client: {e}"))
                })?,
            base_url,
        })
    }
    pub fn post<T: Serialize + ?Sized>(
        &self,
        route: &str,
        body: &T,
    ) -> Result<reqwest::Response, TtsError> {
        let res = self
            .client
            .post(self.base_url.clone() + route)
            .json(body)
            .send();
        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(resp);
                }
                macro_rules! msg {
                    ($msg:literal) => {
                        if let Ok(r) = resp.json::<ErrorResponse>() {
                            format!(concat!($msg, ": {}"), r.err_msg)
                        } else {
                            $msg.to_string()
                        }
                    };
                    ($msg:expr) => {{
                        let mut out = $msg.to_string();
                        if let Ok(r) = resp.json::<ErrorResponse>() {
                            out.push_str(": ");
                            out.push_str(&r.err_msg);
                        }
                        out
                    }};
                }
                Err(match resp.status() {
                    StatusCode::BAD_REQUEST => {
                        if let Ok(e) = resp.json::<ErrorResponse>() {
                            match e.err_code.as_str() {
                                "INVALID_JSON" => TtsError::InternalError(format!(
                                    "Invalid request JSON: {}",
                                    e.err_msg
                                )),
                                "INVALID_QUERY_PARAMETER" => TtsError::InternalError(e.err_msg),
                                "Bad Request" => {
                                    // of course we couldn't get more information from the code, that'd be silly
                                    match e.err_msg.as_str() {
                                        "Bad Request: No such model/language/tier combination found." => TtsError::ModelNotFound("Model/language/tier combination not found".to_string()),
                                        "Input text contains no characters." => TtsError::InvalidText("Empty input text".to_string()),
                                        msg => {
                                            if msg.starts_with("Unsupported audio format: ") {
                                                TtsError::UnsupportedOperation(e.err_msg)
                                            } else {
                                                TtsError::InternalError("Unknown client-side error".to_string())
                                            }
                                        }
                                    }
                                }
                                _ => TtsError::InternalError(format!(
                                    "Invalid request: {}",
                                    e.err_msg
                                )),
                            }
                        } else {
                            TtsError::InternalError("Invalid request".to_string())
                        }
                    }
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                        TtsError::Unauthorized(msg!("Invalid credentials"))
                    }
                    StatusCode::NOT_FOUND => todo!(),
                    StatusCode::PAYLOAD_TOO_LARGE => {
                        const DEFAULT_MAX: u32 = 1000;
                        let len = resp.json::<ErrorResponse>().map_or(DEFAULT_MAX, |e| {
                            e.err_msg
                                .strip_prefix("Input text exceeds maximum character limit of ")
                                .map_or(DEFAULT_MAX, |l| l.parse().unwrap_or(DEFAULT_MAX))
                        });
                        TtsError::TextTooLong(len)
                    }
                    StatusCode::TOO_MANY_REQUESTS => TtsError::RateLimited(5),
                    s => {
                        if s.is_server_error() {
                            TtsError::NetworkError(msg!("Server error"))
                        } else {
                            let kind = if s.is_client_error() {
                                "Client-side error"
                            } else {
                                "Unknown failure"
                            };
                            TtsError::InternalError(msg!(kind))
                        }
                    }
                })
            }
            Err(err) => Err(if err.is_timeout() {
                TtsError::NetworkError("Timeout on connect".to_string())
            } else if err.is_request() {
                TtsError::NetworkError("Error connecting to client".to_string())
            } else {
                TtsError::InternalError(err.to_string())
            }),
        }
    }
}
