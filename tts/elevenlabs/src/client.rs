use golem_tts::config::{get_elevenlabs_api_key, get_elevenlabs_base_url};
use golem_tts::durability::{backoff_delay, max_retries};
use golem_tts::error::error_from_status;
use golem_tts::golem::tts::types::TtsError;
use reqwest::{header, Client as ReqwestClient, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceSettings {
    pub stability: f32,
    pub similarity_boost: f32,
    pub style: Option<f32>,
    pub use_speaker_boost: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceSharing {
    pub status: String,
    pub history_item_sample_id: Option<String>,
    pub original_voice_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceVerification {
    pub verification_attempts: Option<Vec<Value>>, // Can be complex, using Value
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub name: String,
    pub preview_url: Option<String>,
    pub settings: Option<VoiceSettings>,
    pub sharing: Option<VoiceSharing>,
    pub high_quality_base_model_ids: Vec<String>,
    pub safety_control: Option<String>,
    pub voice_verification: Option<VoiceVerification>,
    pub category: String,
    pub labels: HashMap<String, String>,
    pub available_for_tiers: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VoicesResponse {
    voices: Vec<VoiceInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoiceDetails {
    pub voice_id: String,
    pub name: String,
    pub description: Option<String>,
    pub labels: HashMap<String, String>,
    pub category: String,
    pub settings: Option<VoiceSettings>,
    pub available_for_tiers: Vec<String>,
}

// endregion: --- End of API Data Structures ---

/// The main client for interacting with the ElevenLabs API.
#[derive(Clone)]
pub struct ElevenLabsClient {
    client: ReqwestClient,
    base_url: String,
}

impl ElevenLabsClient {
    /// Creates a new client, configured from environment variables.
    pub fn new() -> Result<Self, TtsError> {
        let api_key = get_elevenlabs_api_key()?;
        let base_url = get_elevenlabs_base_url();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "xi-api-key",
            header::HeaderValue::from_str(&api_key).map_err(|_| {
                TtsError::InvalidConfiguration("Invalid ElevenLabs API key".to_string())
            })?,
        );

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(golem_tts::config::get_timeout_seconds()))
            .build()
            .map_err(|e| TtsError::InvalidConfiguration(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, base_url })
    }

    /// Helper to send a request with exponential backoff retry logic.
    async fn send_request_with_retry(
        &self,
        builder: RequestBuilder,
    ) -> Result<reqwest::Response, TtsError> {
        let max_retries = max_retries();
        for attempt in 0..max_retries {
            // We must clone the builder to reuse it in case of a retry.
            if let Some(cloned_builder) = builder.try_clone() {
                let res = cloned_builder.send().await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            return Ok(response);
                        }
                        // For client errors (4xx), don't retry.
                        if response.status().is_client_error() {
                            let status = response.status();
                            let body = response.text().await.ok();
                            return Err(error_from_status(status, body));
                        }
                        // For server errors (5xx), proceed to retry.
                    }
                    Err(e) if e.is_connect() || e.is_timeout() => {
                        // Retry on connection or timeout errors.
                    }
                    Err(e) => {
                        // For other errors, fail immediately.
                        return Err(TtsError::NetworkError(e.to_string()));
                    }
                }
            } else {
                return Err(TtsError::InternalError(
                    "Failed to clone request for retry".to_string(),
                ));
            }

        }

        Err(TtsError::ServiceUnavailable(
            "Service failed to respond after multiple retries".to_string(),
        ))
    }

    /// Fetches all available voices from the API.
    pub async fn get_voices(&self) -> Result<Vec<VoiceInfo>, TtsError> {
        let url = format!("{}/v1/voices", self.base_url);
        let builder = self.client.get(&url);
        let response = self.send_request_with_retry(builder).await?;
        let voices_response = response
            .json::<VoicesResponse>()
            .await
            .map_err(|e| TtsError::InternalError(format!("Failed to parse voices response: {}", e)))?;
        Ok(voices_response.voices)
    }

    /// Fetches detailed information for a single voice.
    pub async fn get_voice(&self, voice_id: &str) -> Result<VoiceDetails, TtsError> {
        let url = format!("{}/v1/voices/{}", self.base_url, voice_id);
        let builder = self.client.get(&url);
        let response = self.send_request_with_retry(builder).await?;
        response
            .json::<VoiceDetails>()
            .await
            .map_err(|e| TtsError::InternalError(format!("Failed to parse voice details: {}", e)))
    }
    
    /// Performs the text-to-speech synthesis.
    #[allow(clippy::too_many_arguments)]
    pub async fn text_to_speech(
        &self,
        voice_id: &str,
        text: &str,
        model_id: &str,
        voice_settings: Option<&VoiceSettings>,
        format_string: &str,
    ) -> Result<(bytes::Bytes, Option<String>), TtsError> {
        let url = format!("{}/v1/text-to-speech/{}?output_format={}", self.base_url, voice_id, format_string);
        
        let mut payload = serde_json::json!({
            "text": text,
            "model_id": model_id,
        });

        if let Some(settings) = voice_settings {
            payload["voice_settings"] = serde_json::to_value(settings)
                .map_err(|e| TtsError::InternalError(format!("Failed to serialize voice settings: {}", e)))?;
        }
        
        let builder = self.client.post(&url).json(&payload);
        let response = self.send_request_with_retry(builder).await?;

        let request_id = response.headers()
            .get("request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| TtsError::SynthesisFailed(format!("Failed to read audio stream: {}", e)))?;

        Ok((audio_data, request_id))
    }
}