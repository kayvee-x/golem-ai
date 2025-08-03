mod client;
mod conversions;

// Use shared components from the `golem-tts` crate and generated WIT bindings.
use golem_tts::golem::tts::{advanced, synthesis, streaming, types, voices};
use golem_tts::{export_tts, init_logging};

use crate::client::{ElevenLabsClient, VoiceInfo};
use crate::conversions::VoiceData;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use types::TtsError;

// A helper macro to reduce boilerplate when creating and running a Tokio runtime
// to bridge the async client calls with the synchronous WIT world.
macro_rules! block_on {
    ($async_expr:expr) => {
        tokio::runtime::Runtime::new()
            .map_err(|e| TtsError::InternalError(format!("Failed to create Tokio runtime: {}", e)))?
            .block_on($async_expr)
    };
}

// region: --- In-Memory Resource Management ---

// A thread-local, in-memory store for WIT resources. The key is a unique u64 ID.
// `RefCell` is used for interior mutability in the single-threaded WASM environment.
thread_local! {
    static VOICE_RESOURCES: RefCell<HashMap<u64, VoiceData>> = RefCell::new(HashMap::new());
    static VOICE_RESULTS_RESOURCES: RefCell<HashMap<u64, Vec<VoiceInfo>>> = RefCell::new(HashMap::new());
    // For future streaming implementation:
    // static STREAM_RESOURCES: RefCell<HashMap<u64, StreamState>> = RefCell::new(HashMap::new());
}

// Simple atomic counters to generate unique IDs for each new resource.
static NEXT_VOICE_ID: Mutex<u64> = Mutex::new(0);
static NEXT_VOICE_RESULTS_ID: Mutex<u64> = Mutex::new(0);

fn new_resource_id(mutex: &Mutex<u64>) -> u64 {
    let mut guard = mutex.lock().unwrap();
    let id = *guard;
    *guard += 1;
    id
}

// The main component struct. It is stateless itself; all state is managed
// in Golem's durability layer or in the thread-local resource stores above.
struct TtsComponent;

impl TtsComponent {
    /// Helper to instantiate the API client, centralizing error handling.
    fn client() -> Result<ElevenLabsClient, TtsError> {
        ElevenLabsClient::new()
    }
}

// Export the WIT world, binding the `TtsComponent` to the WIT interfaces.
export_tts!(TtsComponent);

// endregion: --- Resource Management ---

// --- Implementation of the `voices` interface ---
impl voices::Host for TtsComponent {
    fn list_voices(
        filter: Option<voices::VoiceFilter>,
    ) -> Result<voices::VoiceResults, TtsError> {
        init_logging();
        log::info!("Listing available voices with filter: {:?}", filter);

        let all_voices = block_on!(Self::client()?.get_voices())?;

        // Apply filtering logic here. This is a basic example.
        let filtered_voices: Vec<VoiceInfo> = all_voices
            .into_iter()
            .filter(|v| {
                if let Some(f) = &filter {
                    if let Some(lang) = &f.language {
                        // A more robust implementation would use conversions::extract_language
                        if !v.name.to_lowercase().contains(&lang.to_lowercase()) {
                            return false;
                        }
                    }
                    if let Some(gender) = f.gender {
                        if conversions::extract_gender(&v.labels) != gender {
                            return false;
                        }
                    }
                    if let Some(q) = f.quality {
                        if conversions::map_quality(&v.category, &v.available_for_tiers) != q {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();

        let id = new_resource_id(&NEXT_VOICE_RESULTS_ID);
        VOICE_RESULTS_RESOURCES.with_borrow_mut(|map| {
            map.insert(id, filtered_voices);
        });

        Ok(voices::VoiceResults::new(id))
    }

    fn get_voice(voice_id: String) -> Result<voices::Voice, TtsError> {
        init_logging();
        log::info!("Getting details for voice: {}", voice_id);

        let details = block_on!(Self::client()?.get_voice(&voice_id))?;
        let voice_data = conversions::convert_voice_details(&details);
        let id = new_resource_id(&NEXT_VOICE_ID);

        VOICE_RESOURCES.with_borrow_mut(|map| {
            map.insert(id, voice_data);
        });

        Ok(voices::Voice::new(id))
    }

    // Other functions like `search-voices` and `list-languages` would be implemented here.
    // For brevity, they are omitted but would follow a similar pattern.
}

// --- Implementation for the `voice` resource methods ---
impl voices::HostVoice for TtsComponent {
    fn get_id(&self, self_resource: voices::Voice) -> String {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().id.clone())
    }

    fn get_name(&self, self_resource: voices::Voice) -> String {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().name.clone())
    }

    fn get_provider_id(&self, self_resource: voices::Voice) -> Option<String> {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().provider_id.clone())
    }

    fn get_language(&self, self_resource: voices::Voice) -> String {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().language.clone())
    }
    
    // Implement all other `HostVoice` getters similarly...
    fn get_gender(&self, self_resource: voices::Voice) -> types::VoiceGender {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().gender)
    }

    fn get_quality(&self, self_resource: voices::Voice) -> types::VoiceQuality {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().quality)
    }

    fn supports_ssml(&self, _self_resource: voices::Voice) -> bool { true }

    fn get_supported_formats(&self, self_resource: voices::Voice) -> Vec<types::AudioFormat> {
        VOICE_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().supported_formats.clone())
    }
    
    fn update_settings(&self, _self_resource: voices::Voice, _settings: types::VoiceSettings) -> Result<(), TtsError> {
        Err(TtsError::UnsupportedOperation("Updating voice settings directly is not supported by ElevenLabs. Provide settings at synthesis time.".to_string()))
    }
    
    fn delete(&self, _self_resource: voices::Voice) -> Result<(), TtsError> {
        Err(TtsError::UnsupportedOperation("Deleting premade voices is not supported.".to_string()))
    }

    fn drop(&mut self, self_resource: voices::Voice) {
        log::info!("Dropping voice resource: {}", self_resource.id);
        VOICE_RESOURCES.with_borrow_mut(|map| map.remove(&self_resource.id));
    }
}

// --- Implementation for the `voice-results` resource methods ---
impl voices::HostVoiceResults for TtsComponent {
    fn has_more(&self, self_resource: voices::VoiceResults) -> bool {
        // This simple implementation fetches all results at once.
        !VOICE_RESULTS_RESOURCES.with_borrow(|map| map.get(&self_resource.id).unwrap().is_empty())
    }
    
    fn get_next(&self, self_resource: voices::VoiceResults) -> Result<Vec<voices::VoiceInfo>, TtsError> {
        // Since we fetch all at once, this returns everything and clears the resource.
        let mut results = VOICE_RESULTS_RESOURCES.with_borrow_mut(|map| {
            map.remove(&self_resource.id).unwrap_or_default()
        });
        Ok(results.drain(..).map(|v| conversions::convert_voice_info(&v)).collect())
    }

    fn get_total_count(&self, self_resource: voices::VoiceResults) -> Option<u32> {
        VOICE_RESULTS_RESOURCES.with_borrow(|map| {
            map.get(&self_resource.id).map(|v| v.len() as u32)
        })
    }

    fn drop(&mut self, self_resource: voices::VoiceResults) {
        VOICE_RESULTS_RESOURCES.with_borrow_mut(|map| map.remove(&self_resource.id));
    }
}

// --- Implementation of the `synthesis` interface ---
impl synthesis::Host for TtsComponent {
    fn synthesize(
        input: types::TextInput,
        voice: voices::Voice,
        options: Option<synthesis::SynthesisOptions>,
    ) -> Result<types::SynthesisResult, TtsError> {
        init_logging();
        log::info!("Synthesizing text for voice resource ID: {}", voice.id);

        conversions::validate_text_input(&input)?;
        
        let (voice_id, default_settings) = VOICE_RESOURCES.with_borrow(|map| {
            map.get(&voice.id)
               .map(|data| (data.id.clone(), data.settings.clone()))
               .ok_or_else(|| TtsError::VoiceNotFound(format!("Resource ID {} is invalid or has been dropped", voice.id)))
        })?;

        let final_settings = options.as_ref()
            .and_then(|o| o.voice_settings.as_ref())
            .map(conversions::convert_to_el_voice_settings)
            .or(default_settings);

        let model_id = conversions::select_model_for_effects(options.as_ref().and_then(|o| o.audio_effects.as_ref()));
        let format_str = options.as_ref()
            .and_then(|o| o.audio_config.as_ref())
            .map_or(Ok("mp3_44100_128".to_string()), |ac| conversions::convert_audio_format(ac.format))?;

        let (audio_data, req_id) = block_on!(Self::client()?.text_to_speech(
            &voice_id, &input.content, &model_id, final_settings.as_ref(), &format_str
        ))?;
        
        let metadata = conversions::create_synthesis_metadata(&audio_data, input.content.len() as u32, req_id);
        
        Ok(types::SynthesisResult { audio_data: audio_data.to_vec(), metadata })
    }

    fn synthesize_batch(
        _inputs: Vec<types::TextInput>,
        _voice: voices::Voice,
        _options: Option<synthesis::SynthesisOptions>,
    ) -> Result<Vec<types::SynthesisResult>, TtsError> {
        Err(TtsError::UnsupportedOperation("Batch synthesis is not directly supported; please call synthesize multiple times.".to_string()))
    }
}

// --- Implementation of the `advanced` interface ---
impl advanced::Host for TtsComponent {
    fn generate_sound_effect(
        description: String,
        duration_seconds: Option<f32>,
    ) -> Result<Vec<u8>, TtsError> {
        init_logging();
        log::info!("Generating sound effect for description: {}", description);

        let audio_data = block_on!(Self::client()?.sound_generation(&description, duration_seconds))?;

        Ok(audio_data.to_vec())
    }

    // The features below are not a good fit for ElevenLabs' API structure or require
    // more complex state/interaction than is suitable for a simple component.
    fn create_voice_clone(
        _name: String,
        _audio_samples: Vec<advanced::AudioSample>,
        _description: Option<String>,
    ) -> Result<voices::Voice, TtsError> {
        Err(TtsError::UnsupportedOperation("Programmatic voice cloning via this interface is not yet implemented.".to_string()))
    }

    fn create_lexicon(
        _name: String,
        _language: types::LanguageCode,
        _entries: Option<Vec<advanced::PronunciationEntry>>,
    ) -> Result<advanced::PronunciationLexicon, TtsError> {
        Err(TtsError::UnsupportedOperation("ElevenLabs uses Pronunciation Dictionaries, which have a different API. This WIT interface is better suited for providers like AWS Polly.".to_string()))
    }

    fn synthesize_long_form(
        _content: String,
        _voice: voices::Voice,
        _output_location: String,
        _chapter_breaks: Option<Vec<u32>>,
    ) -> Result<advanced::LongFormOperation, TtsError> {
        Err(TtsError::UnsupportedOperation("ElevenLabs does not support asynchronous long-form synthesis tasks. For long content, please split the text and call `synthesize` multiple times.".to_string()))
    }
}


// --- Implementation of the `streaming` interface (Placeholder) ---
impl streaming::Host for TtsComponent {
    fn create_stream(
        _voice: voices::Voice,
        _options: Option<synthesis::SynthesisOptions>,
    ) -> Result<streaming::SynthesisStream, TtsError> {
        Err(TtsError::UnsupportedOperation("Real-time streaming synthesis via WebSockets is not yet implemented in this component.".to_string()))
    }
}