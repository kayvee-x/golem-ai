use crate::client::{ElevenLabsAudioResponse, ElevenLabsVoice};
use tts::types::{TtsAudio, TtsRequest, TtsVoice};
use base64::{engine::general_purpose, Engine};

/// Converts a `TtsRequest` into the JSON payload ElevenLabs expects.
pub fn tts_request_to_payload(req: &TtsRequest) -> serde_json::Value {
    serde_json::json!({
        "text": req.text,
        "model_id": req.model_id.clone().unwrap_or_else(|| "eleven_monolingual_v1".to_string()),
        "voice_settings": {
            "stability": req.stability.unwrap_or(0.5),
            "similarity_boost": req.similarity_boost.unwrap_or(0.75)
        }
    })
}

/// Converts an ElevenLabs API voice response to a generic `TtsVoice`.
pub fn elevenlabs_voice_to_tts_voice(voice: ElevenLabsVoice) -> TtsVoice {
    TtsVoice {
        id: voice.voice_id,
        name: voice.name,
        description: voice.labels.map(|l| l.join(", ")),
        language: voice.language,
        gender: voice.gender,
    }
}

/// Converts a list of ElevenLabs voices to a list of TtsVoice objects.
pub fn elevenlabs_voices_to_tts_voices(voices: Vec<ElevenLabsVoice>) -> Vec<TtsVoice> {
    voices
        .into_iter()
        .map(elevenlabs_voice_to_tts_voice)
        .collect()
}

/// Converts the raw ElevenLabs audio response to a usable `TtsAudio` object.
pub fn audio_response_to_tts_audio(response: ElevenLabsAudioResponse) -> TtsAudio {
    let base64_audio = general_purpose::STANDARD.encode(response.audio_data);

    TtsAudio {
        audio_base64: base64_audio,
        content_type: "audio/mpeg".to_string(),
    }
}