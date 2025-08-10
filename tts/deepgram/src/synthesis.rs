use super::*;
use crate::client::Client;
use crate::voices::Voice;
use golem_tts::exports::golem::tts::synthesis::*;
use golem_tts::golem::tts::types::{AudioFormat, SynthesisMetadata, TextType, TtsError};
use serde::Serialize;

#[derive(Serialize)]
struct Text<'a> {
    text: &'a str,
}

fn synthesize_impl(
    input: &TextInput,
    voice: &Voice,
    options: Option<&SynthesisOptions>,
) -> Result<SynthesisResult, TtsError> {
    use std::fmt::Write;
    if input.text_type == TextType::Ssml {
        return Err(TtsError::UnsupportedOperation(
            "SSML text is not supported".to_string(),
        ));
    }
    let mut route = "/v1/speak?".to_string();
    let mut model = "aura".to_string();
    if let Some(opts) = options {
        if let Some(v) = &opts.model_version {
            model.push('-');
            model.push_str(v);
        } else {
            model.push_str("-2");
        }
        model.push_str("thalia");
        if let Some(lang) = &input.language {
            model.push('-');
            model.push_str(lang);
        } else {
            model.push_str("-en");
        }
        if let Some(audio) = opts.audio_config {
            if let Some(bit_rate) = audio.bit_rate {
                let _ = write!(route, "bit_rate={bit_rate}&");
            }
            let fmt = match audio.format {
                AudioFormat::Aac => "encoding=aac&",
                AudioFormat::Alaw => "encoding=alaw&",
                AudioFormat::Flac => "encoding=flac&",
                AudioFormat::Mp3 => "encoding=mp3&",
                AudioFormat::Mulaw => "encoding=mulaw&",
                AudioFormat::OggOpus => "encoding=opus&",
                AudioFormat::Pcm | AudioFormat::Wav => "encoding=linear16&",
            };
            route.push_str(fmt);
        }
    }
    route.push_str("model=");
    route.push_str(&model);
    let client = Client::new()?;
    let resp = client.post(
        &route,
        &Text {
            text: &input.content,
        },
    )?;
    let id = resp
        .headers()
        .get("dg-request-id")
        .map_or(String::new(), |v| {
            String::from_utf8_lossy(v.as_bytes()).into_owned()
        });
    let mut words = 0;
    let mut chars = 0;
    let mut in_whitespace = true;
    for ch in input.content.chars() {
        if ch.is_whitespace() {
            in_whitespace = true;
        } else if in_whitespace {
            in_whitespace = false;
            words += 1;
        }
        chars += 1;
    }
    let audio_data: Vec<u8> = match resp.bytes() {
        Ok(b) => b.into(),
        Err(err) => return Err(TtsError::InternalError(err.to_string())),
    };
    let size = audio_data.len();
    Ok(SynthesisResult {
        audio_data,
        metadata: SynthesisMetadata {
            duration_seconds: 0.0,
            character_count: chars,
            word_count: words,
            audio_size_bytes: size as _,
            request_id: id,
            provider_info: None,
        },
    })
}

impl Guest for DeepgramComponent {
    fn synthesize(
        input: TextInput,
        voice: VoiceBorrow<'_>,
        options: Option<SynthesisOptions>,
    ) -> Result<SynthesisResult, TtsError> {
        synthesize_impl(&input, voice.get(), options.as_ref())
    }
    fn synthesize_batch(
        inputs: Vec<TextInput>,
        voice: VoiceBorrow<'_>,
        options: Option<SynthesisOptions>,
    ) -> Result<Vec<SynthesisResult>, TtsError> {
        inputs
            .iter()
            .map(|i| synthesize_impl(i, voice.get(), options.as_ref()))
            .collect()
    }
    fn get_timing_marks(
        input: TextInput,
        voice: VoiceBorrow<'_>,
    ) -> Result<Vec<TimingInfo>, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Timing marks aren't yet implemented".to_string(),
        ))
    }
    fn validate_input(
        input: TextInput,
        voice: VoiceBorrow<'_>,
    ) -> Result<ValidationResult, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Input validation isn't yet implemented".to_string(),
        ))
    }
}
