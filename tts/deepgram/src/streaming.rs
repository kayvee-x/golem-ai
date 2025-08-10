#![allow(unused_variables)]

use super::*;
use golem_tts::exports::golem::tts::streaming::{
    SynthesisStream as GolemSynthesisStream, VoiceConversionStream as GolemConversionStream, *,
};

pub struct SynthesisStream {}
impl GuestSynthesisStream for SynthesisStream {
    fn send_text(&self, input: TextInput) -> Result<(), TtsError> {
        unreachable!()
    }
    fn finish(&self) -> Result<(), TtsError> {
        unreachable!()
    }
    fn receive_chunk(&self) -> Result<Option<AudioChunk>, TtsError> {
        unreachable!()
    }
    fn has_pending_audio(&self) -> bool {
        unreachable!()
    }
    fn get_status(&self) -> StreamStatus {
        unreachable!()
    }
    fn close(&self) {
        unreachable!()
    }
}

pub struct ConversionStream {}
impl GuestVoiceConversionStream for ConversionStream {
    fn send_audio(&self, audio_data: Vec<u8>) -> Result<(), TtsError> {
        unreachable!()
    }
    fn receive_converted(&self) -> Result<Option<AudioChunk>, TtsError> {
        unreachable!()
    }
    fn finish(&self) -> Result<(), TtsError> {
        unreachable!()
    }
    fn close(&self) {
        unreachable!()
    }
}

impl Guest for DeepgramComponent {
    type SynthesisStream = SynthesisStream;
    type VoiceConversionStream = ConversionStream;
    fn create_stream(
        voice: VoiceBorrow<'_>,
        options: Option<SynthesisOptions>,
    ) -> Result<GolemSynthesisStream, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Streaming is not yet implemented".to_string(),
        ))
    }
    fn create_voice_conversion_stream(
        target_voice: VoiceBorrow<'_>,
        options: Option<SynthesisOptions>,
    ) -> Result<GolemConversionStream, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Streaming is not yet implemented".to_string(),
        ))
    }
}
