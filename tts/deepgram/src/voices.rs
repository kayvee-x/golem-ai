use super::*;
use golem_tts::exports::golem::tts::voices::{
    Voice as GolemVoice, VoiceResults as GolemVoiceResults, *,
};

pub struct VoiceData {
    id: &'static str,
    name: &'static str,
    spanish: bool,
    gender: VoiceGender,
    tags: &'static [&'static str],
    uses: &'static [&'static str],
}
impl VoiceData {
    pub fn info(&self) -> VoiceInfo {
        VoiceInfo {
            id: self.id.to_string(),
            name: self.name.to_string(),
            language: if self.spanish {
                "es".to_string()
            } else {
                "en".to_string()
            },
            additional_languages: Vec::new(),
            gender: self.gender,
            quality: VoiceQuality::Standard,
            description: Some(self.tags.join(", ")),
            provider: "Deepgram".to_string(),
            sample_rate: 0,
            is_custom: false,
            is_cloned: false,
            preview_url: None,
            use_cases: self.uses.iter().map(|u| u.to_string()).collect(),
        }
    }
    fn matches(&self, f: &VoiceFilter) -> bool {
        f.gender.is_none_or(|x| x == self.gender)
            && f.language
                .as_ref()
                .is_none_or(|x| if self.spanish { x == "es" } else { x == "en" })
            && f.supports_ssml != Some(true)
    }
}

pub static VOICES: &[VoiceData] = &[];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice(u8);
impl Voice {
    pub fn resolve(&self) -> &'static VoiceData {
        &VOICES[self.0 as usize]
    }
}
impl GuestVoice for Voice {
    fn get_id(&self) -> String {
        self.resolve().id.to_string()
    }
    fn get_name(&self) -> String {
        self.resolve().name.to_string()
    }
    fn get_provider_id(&self) -> Option<String> {
        None
    }
    fn get_language(&self) -> LanguageCode {
        if self.resolve().spanish {
            "es".to_string()
        } else {
            "en".to_string()
        }
    }
    fn get_additional_languages(&self) -> Vec<LanguageCode> {
        Vec::new()
    }
    fn get_gender(&self) -> VoiceGender {
        self.resolve().gender
    }
    fn get_quality(&self) -> VoiceQuality {
        VoiceQuality::Standard
    }
    fn get_description(&self) -> Option<String> {
        Some(self.resolve().tags.join(", "))
    }
    fn supports_ssml(&self) -> bool {
        false
    }
    fn get_sample_rates(&self) -> Vec<u32> {
        Vec::new()
    }
    fn get_supported_formats(&self) -> Vec<AudioFormat> {
        vec![AudioFormat::Mp3]
    }
    fn update_settings(&self, _settings: VoiceSettings) -> Result<(), TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Modifying voices isn't supported".to_string(),
        ))
    }
    fn clone(&self) -> Result<GolemVoice, TtsError> {
        Ok(GolemVoice::new(*self))
    }
    fn delete(&self) -> Result<(), TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Deleting voices isn't supported".to_string(),
        ))
    }
    fn preview(&self, _text: String) -> Result<Vec<u8>, TtsError> {
        Err(TtsError::UnsupportedOperation(
            "Previewing voices isn't supported".to_string(),
        ))
    }
}

pub struct VoiceResults(Option<Vec<Voice>>);
impl GuestVoiceResults for VoiceResults {
    fn get_next(&self) -> Result<Vec<VoiceInfo>, TtsError> {
        Ok(if let Some(indices) = &self.0 {
            indices.iter().map(|v| v.resolve().info()).collect()
        } else {
            VOICES.iter().map(|d| d.info()).collect()
        })
    }
    fn get_total_count(&self) -> Option<u32> {
        Some(self.0.as_ref().map_or(VOICES.len(), |s| s.len()) as _)
    }
    fn has_more(&self) -> bool {
        false
    }
}

impl Guest for DeepgramComponent {
    type Voice = Voice;
    type VoiceResults = VoiceResults;

    fn get_voice(voice_id: String) -> Result<GolemVoice, TtsError> {
        VOICES.iter().position(|v| voice_id == v.id).map_or_else(
            || Err(TtsError::VoiceNotFound(voice_id)),
            |i| Ok(GolemVoice::new(Voice(i as _))),
        )
    }
    fn list_languages() -> Result<Vec<LanguageInfo>, TtsError> {
        Ok(vec![
            LanguageInfo {
                code: "en".to_string(),
                name: "English".to_string(),
                native_name: "English".to_string(),
                voice_count: 0,
            },
            LanguageInfo {
                code: "es".to_string(),
                name: "Spanish".to_string(),
                native_name: "Español".to_string(),
                voice_count: 0,
            },
        ])
    }
    fn list_voices(filter: Option<VoiceFilter>) -> Result<GolemVoiceResults, TtsError> {
        Ok(GolemVoiceResults::new(VoiceResults(filter.map(|f| {
            VOICES
                .iter()
                .enumerate()
                .filter(|(_, v)| v.matches(&f))
                .map(|x| Voice(x.0 as _))
                .collect()
        }))))
    }
    fn search_voices(
        mut query: String,
        filter: Option<VoiceFilter>,
    ) -> Result<Vec<VoiceInfo>, TtsError> {
        query.make_ascii_lowercase();
        Ok(VOICES
            .iter()
            .filter(|v| filter.as_ref().is_none_or(|f| v.matches(f)) && v.name.contains(&query))
            .map(|v| v.info())
            .collect())
    }
}
