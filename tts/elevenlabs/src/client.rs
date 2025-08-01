use reqwest::{Client, StatusCode};
use crate::conversions::to_ssml;
use crate::error;
use crate::config;
use crate::bindings::types::{AudioResult, TtsError};

const BASE_URL: &str = "https://api.elevenlabs.io/v1";

pub async fn generate_speech(text: &str) -> Result<AudioResult, TtsError> {
    let api_key = config::validate_config_key("ELEVENLABS_API_KEY")?;
    let voice_id = config::get_default_voice();
    let ssml = to_ssml(text);
    let url = format!("{}/text-to-speech/{}", BASE_URL, voice_id);

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"text": "{}"}}"#, ssml)) // optional: improve JSON escape
        .send()
        .await
        .map_err(|e| error::from_reqwest_error("Sending request", e))?;

    let status = resp.status();
    let bytes = resp.bytes().await.map_err(|e| error::from_reqwest_error("Reading response", e))?;

    if !status.is_success() {
        return Err(error::error_from_status(status, Some(String::from_utf8_lossy(&bytes).to_string())));
    }

    Ok(AudioResult {
        audio_bytes: bytes.to_vec(),
    })
}