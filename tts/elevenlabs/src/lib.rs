mod client;
mod conversions;

use crate::client::ElevenLabsClient;
use crate::conversions::{api_response_to_tts_result, voice_settings_to_api_params};

use golem_rust::wasm_rpc::Pollable;
use golem_tts::config::with_config_key;
use golem_tts::durability::{DurableTts, ExtendedGuestTts};
use log::trace;
use std::cell::{Cell, RefCell};

/// Streamed TTS response (not implemented for ElevenLabs in this example)
struct ElevenLabsTtsStream;

impl GuestTtsStream for ElevenLabsTtsStream {
    fn get_next(&self) -> Option<Vec<u8>> {
        Some(vec![]) // Stub
    }

    fn blocking_get_next(&self) -> Vec<u8> {
        self.get_next().unwrap_or_default()
    }
}

/// Main ElevenLabs TTS Component
struct ElevenLabsComponent;

impl ElevenLabsComponent {
    const API_KEY_ENV_VAR: &'static str = "ELEVENLABS_API_KEY";
    const VOICE_ID_ENV_VAR: &'static str = "ELEVENLABS_VOICE_ID";

    fn create_client() -> Result<ElevenLabsClient, TtsError> {
        with_config_key(
            &[Self::API_KEY_ENV_VAR, Self::VOICE_ID_ENV_VAR],
            |x| x,
            |keys| {
                if keys.len() < 2 || keys[0].is_empty() || keys[1].is_empty() {
                    return Err(TtsError::Internal("Missing ElevenLabs credentials".into()));
                }

                Ok(ElevenLabsClient::new(keys[0].clone(), keys[1].clone()))
            },
        )
    }
}

impl GuestTts for ElevenLabsComponent {
    type TtsStream = ElevenLabsTtsStream;

    fn speak(text: String, options: Option<TtsOptions>) -> Result<TtsResult, TtsError> {
        let client = Self::create_client()?;
        let params = voice_settings_to_api_params(options);
        let api_response = client.speak(text, params)?;

        api_response_to_tts_result(api_response)
    }

    fn stream(text: String, options: Option<TtsOptions>) -> Result<TtsStream, TtsError> {
        trace!("Streaming is not supported for ElevenLabs in this implementation");
        Ok(TtsStream::new(ElevenLabsTtsStream))
    }
}

impl ExtendedGuestTts for ElevenLabsComponent {
    fn unwrapped_stream(_text: String, _options: Option<TtsOptions>) -> Self::TtsStream {
        ElevenLabsTtsStream
    }

    fn subscribe(_stream: &Self::TtsStream) -> Pollable {
        golem_rust::bindings::wasi::clocks::monotonic_clock::subscribe_duration(0)
    }
}

type DurableElevenLabsComponent = DurableTts<ElevenLabsComponent>;

golem_tts::export_tts!(DurableElevenLabsComponent with_types_in golem_tts);
