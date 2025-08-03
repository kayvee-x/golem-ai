use crate::client::{VoiceInfo, VoiceDetails, VoiceSettings as ELVoiceSettings};
use golem_tts::golem::tts::types::{VoiceSettings, TtsError, AudioFormat, VoiceGender, VoiceQuality, SynthesisMetadata, TextType, TextInput}; 
use std::collections::HashMap;


/// Convert ElevenLabs VoiceInfo to WIT VoiceInfo
pub fn convert_voice_info(el_voice: &VoiceInfo) -> VoiceInfo {
    VoiceInfo {
        voice_id: el_voice.voice_id.clone(),
        name: el_voice.name.clone(),
        preview_url: el_voice.preview_url.clone(),
        settings: el_voice.settings.clone(),
        sharing: el_voice.sharing.clone(),
        high_quality_base_model_ids: el_voice.high_quality_base_model_ids.clone(),
        safety_control: el_voice.safety_control.clone(),
        voice_verification: el_voice.voice_verification.clone(),
        category: el_voice.category.clone(), 
        labels: el_voice.labels.clone(), 
        available_for_tiers: el_voice.available_for_tiers.clone(), 
        description: el_voice.description.clone(), 
    }
}

/// Convert ElevenLabs VoiceDetails to WIT Voice resource data
pub fn convert_voice_details(el_voice: &VoiceDetails) -> VoiceData {
    VoiceData {
        id: el_voice.voice_id.clone(),
        name: el_voice.name.clone(),
        provider_id: Some(el_voice.voice_id.clone()),
        language: extract_language(&el_voice.labels).unwrap_or_else(|| "en-US".to_string()),
        additional_languages: extract_additional_languages(&el_voice.labels),
        gender: extract_gender(&el_voice.labels),
        quality: map_quality(&el_voice.category, &el_voice.available_for_tiers),
        description: el_voice.description.clone(),
        supports_ssml: true, // ElevenLabs supports SSML
        sample_rates: vec![22050, 44100], // Common ElevenLabs sample rates
        supported_formats: vec![
            AudioFormat::Mp3,
            AudioFormat::Wav,
            AudioFormat::PcmU8,
            AudioFormat::PcmI16,
            AudioFormat::PcmI24,
            AudioFormat::PcmI32,
            AudioFormat::PcmF32,
            AudioFormat::Ulaw,
            AudioFormat::Alaw,
        ],
        settings: el_voice.settings.as_ref().map(convert_voice_settings),
    }
}

// Internal struct to hold voice data for the resource
pub struct VoiceData {
    pub id: String,
    pub name: String,
    pub provider_id: Option<String>,
    pub language: String,
    pub additional_languages: Vec<String>,
    pub gender: VoiceGender,
    pub quality: VoiceQuality,
    pub description: Option<String>,
    pub supports_ssml: bool,
    pub sample_rates: Vec<u32>,
    pub supported_formats: Vec<AudioFormat>,
    pub settings: Option<VoiceSettings>,
}

/// Convert WIT VoiceSettings to ElevenLabs VoiceSettings
pub fn convert_to_el_voice_settings(settings: &VoiceSettings) -> ELVoiceSettings {
    ELVoiceSettings {
        stability: settings.stability.unwrap_or(0.5),
        similarity_boost: settings.similarity.unwrap_or(0.75),
        style: settings.style,
        use_speaker_boost: Some(true),
    }
}

/// Convert ElevenLabs VoiceSettings to WIT VoiceSettings
fn convert_voice_settings(el_settings: &ELVoiceSettings) -> VoiceSettings {
    VoiceSettings {
        speed: None, // Not directly supported by ElevenLabs
        pitch: None, // Not directly supported by ElevenLabs
        volume: None, // Not directly supported by ElevenLabs
        stability: Some(el_settings.stability),
        similarity: Some(el_settings.similarity_boost),
        style: el_settings.style,
    }
}

/// Convert WIT AudioFormat to ElevenLabs format string
pub fn convert_audio_format(format: AudioFormat) -> Result<String, TtsError> {
    match format {
        AudioFormat::Mp3 => Ok("mp3_44100_128".to_string()),
        AudioFormat::Wav => Ok("wav".to_string()),
        AudioFormat::PcmI16 => Ok("pcm_16000".to_string()),
        AudioFormat::PcmI24 => Ok("pcm_24000".to_string()),
        AudioFormat::PcmF32 => Ok("pcm_44100".to_string()),
        AudioFormat::Ulaw => Ok("ulaw_8000".to_string()),
        _ => Err(TtsError::UnsupportedOperation(format!("Audio format {:?} not supported by ElevenLabs", format))),
    }
}

/// Convert WIT TextType to determine if SSML processing is needed
pub fn should_use_ssml(text_type: TextType) -> bool {
    matches!(text_type, TextType::Ssml)
}

/// Extract language from ElevenLabs labels
fn extract_language(labels: &HashMap<String, String>) -> Option<String> {
    // ElevenLabs uses various label keys for language
    labels.get("language")
        .or_else(|| labels.get("accent"))
        .or_else(|| labels.get("use case"))
        .and_then(|value| {
            match value.to_lowercase().as_str() {
                "american" | "us" => Some("en-US".to_string()),
                "british" | "uk" => Some("en-GB".to_string()),
                "australian" => Some("en-AU".to_string()),
                "spanish" => Some("es-ES".to_string()),
                "french" => Some("fr-FR".to_string()),
                "german" => Some("de-DE".to_string()),
                "italian" => Some("it-IT".to_string()),
                "portuguese" => Some("pt-PT".to_string()),
                "japanese" => Some("ja-JP".to_string()),
                "chinese" => Some("zh-CN".to_string()),
                "korean" => Some("ko-KR".to_string()),
                _ => None,
            }
        })
}

/// Extract additional supported languages
fn extract_additional_languages(labels: &HashMap<String, String>) -> Vec<String> {
    // ElevenLabs voices typically support one primary language
    // This could be enhanced based on actual voice capabilities
    Vec::new()
}

/// Extract gender from voice name and labels
fn extract_gender(labels: &HashMap<String, String>) -> VoiceGender {
    let gender_indicators = labels.values()
        .chain(std::iter::once(&labels.get("gender").unwrap_or(&String::new()).clone()))
        .map(|s| s.to_lowercase())
        .collect::<String>();

    if gender_indicators.contains("male") && !gender_indicators.contains("female") {
        VoiceGender::Male
    } else if gender_indicators.contains("female") {
        VoiceGender::Female
    } else {
        VoiceGender::Neutral
    }
}

/// Map ElevenLabs category to voice quality
fn map_quality(category: &str, available_tiers: &[String]) -> VoiceQuality {
    match category {
        "premade" => {
            if available_tiers.iter().any(|tier| tier.contains("creator") || tier.contains("pro")) {
                VoiceQuality::Premium
            } else {
                VoiceQuality::Standard
            }
        }
        "cloned" => VoiceQuality::Neural,
        "professional" => VoiceQuality::Studio,
        "generated" => VoiceQuality::Neural,
        _ => VoiceQuality::Standard,
    }
}

/// Extract use cases from labels
fn extract_use_cases(labels: &HashMap<String, String>) -> Vec<String> {
    let mut use_cases = Vec::new();
    
    if let Some(use_case) = labels.get("use case") {
        use_cases.push(use_case.clone());
    }
    
    if let Some(accent) = labels.get("accent") {
        use_cases.push(format!("accent: {}", accent));
    }
    
    if let Some(age) = labels.get("age") {
        use_cases.push(format!("age: {}", age));
    }
    
    if use_cases.is_empty() {
        use_cases.push("general".to_string());
    }
    
    use_cases
}

/// Convert synthesis metadata
pub fn create_synthesis_metadata(
    audio_data: &[u8],
    character_count: u32,
    request_id: Option<String>,
) -> SynthesisMetadata {
    let word_count = estimate_word_count(character_count);
    let duration = estimate_duration(character_count);
    
    SynthesisMetadata {
        duration_seconds: duration,
        character_count,
        word_count,
        audio_size_bytes: audio_data.len() as u32,
        request_id: request_id.unwrap_or_else(|| generate_request_id()),
        provider_info: Some("elevenlabs".to_string()),
    }
}

/// Estimate word count from character count
fn estimate_word_count(character_count: u32) -> u32 {
    // Average word length is approximately 5 characters including spaces
    (character_count + 4) / 5
}

/// Estimate audio duration from character count
fn estimate_duration(character_count: u32) -> f32 {
    // Average speaking rate is about 150 words per minute
    // Average word length is 5 characters
    let words = character_count as f32 / 5.0;
    let minutes = words / 150.0;
    minutes * 60.0
}

/// Generate a unique request ID
fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("el_{}", timestamp)
}

/// Rust struct matching the new WIT record audio-effects
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AudioEffects {
    pub telephone_quality: bool,
    pub headphone_optimized: bool,
    pub speaker_optimized: bool,
    pub car_audio_optimized: bool,
    pub noise_reduction: bool,
    pub bass_boost: bool,
    pub treble_boost: bool,
}

/// Map `audio-effects` record to ElevenLabs model selection
pub fn select_model_for_effects(effects: Option<&AudioEffects>) -> String {
    match effects {
        Some(effects) if effects.telephone_quality => "eleven_turbo_v2".to_string(),
        Some(effects) if effects.headphone_optimized => "eleven_multilingual_v2".to_string(),
        _ => "eleven_multilingual_v2".to_string(), // Default high-quality model
    }
}

/// Validate text input for ElevenLabs constraints
pub fn validate_text_input(input: &TextInput) -> Result<(), TtsError> {
    // ElevenLabs has a character limit
    if input.content.len() > 5000 {
        return Err(TtsError::TextTooLong(input.content.len() as u32));
    }
    
    // Check for empty text
    if input.content.trim().is_empty() {
        return Err(TtsError::InvalidText("Text cannot be empty".to_string()));
    }
    
    // Basic SSML validation if needed
    if matches!(input.text_type, TextType::Ssml) {
        validate_ssml(&input.content)?;
    }
    
    Ok(())
}

/// Basic SSML validation
fn validate_ssml(content: &str) -> Result<(), TtsError> {
    // Check for balanced tags - simplified validation
    let open_tags = content.matches('<').count();
    let close_tags = content.matches('>').count();
    
    if open_tags != close_tags {
        return Err(TtsError::InvalidSsml("Unbalanced SSML tags".to_string()));
    }
    
    Ok(())
}