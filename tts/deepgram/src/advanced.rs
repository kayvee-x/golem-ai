#![allow(unused_variables)]

use super::*;
use golem_tts::exports::golem::tts::advanced::*;

pub struct DummyLexicon;
impl GuestPronunciationLexicon for DummyLexicon {
    fn add_entry(&self, word: String, pronunciation: String) -> Result<(), TtsError> {
        unreachable!()
    }
    fn export_content(&self) -> Result<String, TtsError> {
        unreachable!()
    }
    fn get_entry_count(&self) -> u32 {
        unreachable!()
    }
    fn get_language(&self) -> LanguageCode {
        unreachable!()
    }
    fn get_name(&self) -> String {
        unreachable!()
    }
    fn remove_entry(&self, word: String) -> Result<(), TtsError> {
        unreachable!()
    }
}

pub struct DummyOperation;
impl GuestLongFormOperation for DummyOperation {
    fn cancel(&self) -> Result<(), TtsError> {
        unreachable!()
    }
    fn get_progress(&self) -> f32 {
        unreachable!()
    }
    fn get_result(&self) -> Result<LongFormResult, TtsError> {
        unreachable!()
    }
    fn get_status(&self) -> OperationStatus {
        unreachable!()
    }
}

impl Guest for DeepgramComponent {
    type PronunciationLexicon = DummyLexicon;
    type LongFormOperation = DummyOperation;

    fn convert_voice(
        input_audio: Vec<u8>,
        target_voice: VoiceBorrow<'_>,
        preserve_timing: Option<bool>,
    ) -> Result<Vec<u8>, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Voice cloning isn't supported".to_string(),
        ))
    }
    fn create_lexicon(
        name: String,
        language: LanguageCode,
        entries: Option<Vec<PronunciationEntry>>,
    ) -> Result<PronunciationLexicon, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Lexicons aren't supported".to_string(),
        ))
    }
    fn create_voice_clone(
        name: String,
        audio_samples: Vec<AudioSample>,
        description: Option<String>,
    ) -> Result<Voice, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Voice cloning isn't supported".to_string(),
        ))
    }
    fn design_voice(name: String, characteristics: VoiceDesignParams) -> Result<Voice, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Voice design isn't supported".to_string(),
        ))
    }
    fn generate_sound_effect(
        description: String,
        duration_seconds: Option<f32>,
        style_influence: Option<f32>,
    ) -> Result<Vec<u8>, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Sound effects aren't supported".to_string(),
        ))
    }
    fn synthesize_long_form(
        content: String,
        voice: VoiceBorrow<'_>,
        output_location: String,
        chapter_breaks: Option<Vec<u32>>,
    ) -> Result<LongFormOperation, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Long-form synthesis isn't supported".to_string(),
        ))
    }
}
