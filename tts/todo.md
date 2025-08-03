I have attached to this ticket a WIT file that describes a generic interface for text-to-speech operations. This interface can be implemented by various providers, either by emulating features not present in a given provider, utilizing the provider's native support for a feature, or indicating an error if a particular combination is not natively supported by a provider.

The intent of this WIT specification is to allow developers of WASM components (on wasmCloud, Spin, or Golem) to leverage text-to-speech capabilities to build voice-powered applications, accessibility services, and audio content generation systems in a portable and provider-agnostic fashion.

This ticket involves constructing implementations of this WIT interface for the following providers:

ElevenLabs: The leading AI voice synthesis platform with comprehensive voice cloning, real-time streaming, voice conversion, and sound effects generation capabilities.
AWS Polly: Amazon's enterprise text-to-speech service with extensive language support, custom lexicons, speech marks, and asynchronous synthesis for long-form content.
Google Cloud Text-to-Speech: Google's neural voice synthesis service with WaveNet and Neural2 voices, device optimization profiles, and streaming synthesis capabilities.
Deepgram Aura: High-performance real-time TTS with session-based streaming, low-latency neural voices, and conversational AI optimization.
These implementations must be written in Rust and compilable to WASM Components (WASI 0.23 only, since Golem does not yet support WASI 0.3). The standard Rust toolchain for WASM component development can be employed (see cargo component and the Rust examples of components in this and other Golem repositories).

Additionally, these implementations should incorporate custom durability semantics using the Golem durability API and the Golem host API. This approach ensures that durability is managed at the level of individual TTS operations (voice synthesis, streaming session creation, voice cloning, batch processing), providing a higher-level and clearer operation log, which aids in debugging and monitoring. See golem:llm and golem:embed for more details and durable implementations in this same repository.

The final deliverables associated with this ticket are:

ElevenLabs implementation: A WASM Component (WASI 0.23), named tts-elevenlabs.wasm, with a full test suite and custom durability implementation at the level of TTS operations.
AWS Polly implementation: A WASM Component (WASI 0.23), named tts-polly.wasm, with a full test suite and custom durability implementation at the level of TTS operations.
Google Cloud TTS implementation: A WASM Component (WASI 0.23), named tts-google.wasm, with a full test suite and custom durability implementation at the level of TTS operations.
Deepgram Aura implementation: A WASM Component (WASI 0.23), named tts-deepgram.wasm, with a full test suite and custom durability implementation at the level of TTS operations.
Note: If you have a strong recommendation to swap out one or two of these with other popular / common TTS providers (such as Azure Cognitive Services Speech, IBM Watson Text to Speech, or OpenAI TTS), then as long as you get permission beforehand, that's okay with me. However, we definitely need ElevenLabs and AWS Polly.

These components will require runtime configuration, notably API keys, endpoint URLs, authentication credentials, and provider-specific settings. For configuring this information, the components can use environment variables for now (in the future, they will use wasi-runtime-config, but Golem does not support this yet, whereas Golem has good support for environment variables).

Moreover, the Rust components need to be tested within Golem to ensure compatibility with Golem 1.2.x.

This WIT has been designed by examining and comparing the APIs of ElevenLabs, AWS Polly, Google Cloud TTS, Azure Speech Services, OpenAI TTS, and Deepgram Aura. However, given there are no implementations, it is possible the provided WIT is not the optimal abstraction across all these providers. Therefore, deviations from the proposed design can be made. However, to be accepted, any deviation must be fully justified and deemed by Golem core contributors to be an improvement from the original specification.

Implementation Guidelines
Each provider implementation should handle the following key mapping considerations:

Voice Management: Map the unified voice resource to provider-specific voice identifiers, handle voice discovery and metadata appropriately for each provider's voice catalog structure
Audio Format Conversion: Implement native audio format support where available, or provide format conversion for unsupported output formats using audio processing libraries
Streaming Implementation: Utilize native streaming APIs where supported (ElevenLabs, Deepgram), or implement chunk-based synthesis for providers without native streaming support
Authentication Handling: Implement appropriate authentication mechanisms (API keys, OAuth, service accounts) per provider requirements
Feature Availability: Route advanced features (voice cloning, sound effects, speech marks) through provider-native APIs where supported, or return unsupported-operation errors for unavailable features
Error Mapping: Map provider-specific HTTP errors and API responses to the unified tts-error enumeration with appropriate context preservation
Rate Limiting: Handle provider-specific rate limits and quota management, implementing appropriate retry logic and error reporting
Long-form Content: Implement efficient handling of long-form synthesis using provider-native async operations (AWS Polly) or intelligent chunking strategies
Testing Requirements
Each implementation must include comprehensive test suites covering:

Basic synthesis operations (text-to-speech with various voices and configurations)
Voice discovery and metadata retrieval
Streaming synthesis lifecycle (session creation, chunk processing, cleanup)
Advanced feature testing (voice cloning, sound effects, custom pronunciations where supported)
Audio format validation and quality verification
Authentication and authorization scenarios
Error handling for unsupported operations and malformed inputs
Rate limiting and quota management behavior
Connection management and retry logic
Long-form content synthesis (>5000 characters)
Durability semantics verification across operation boundaries
Provider-specific feature utilization (lexicons for Polly, voice settings for ElevenLabs, etc.)
Configuration Requirements
Each implementation should support the following environment variables:

Common Configuration
TTS_PROVIDER_ENDPOINT: Custom endpoint URL (for enterprise/regional deployments)
TTS_PROVIDER_TIMEOUT: Request timeout in seconds (default: 30)
TTS_PROVIDER_MAX_RETRIES: Maximum retry attempts (default: 3)
TTS_PROVIDER_LOG_LEVEL: Logging verbosity (debug, info, warn, error)
Provider-Specific Configuration
ElevenLabs: ELEVENLABS_API_KEY, ELEVENLABS_MODEL_VERSION
AWS Polly: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION, AWS_SESSION_TOKEN
Google Cloud: GOOGLE_APPLICATION_CREDENTIALS, GOOGLE_CLOUD_PROJECT
Deepgram: DEEPGRAM_API_KEY, DEEPGRAM_API_VERSION



package golem:tts@1.0.0;

/// Core types and error handling for universal text-to-speech
interface types {
    /// Comprehensive error types covering all TTS operations
    variant tts-error {
        /// Input validation errors
        invalid-text(string),
        text-too-long(u32),
        invalid-ssml(string),
        unsupported-language(string),
        
        /// Voice and model errors
        voice-not-found(string),
        model-not-found(string),
        voice-unavailable(string),
        
        /// Authentication and authorization
        unauthorized(string),
        access-denied(string),
        
        /// Resource and quota limits
        quota-exceeded(quota-info),
        rate-limited(u32),
        insufficient-credits,
        
        /// Operation errors
        synthesis-failed(string),
        unsupported-operation(string),
        invalid-configuration(string),
        
        /// Service errors
        service-unavailable(string),
        network-error(string),
        internal-error(string),
        
        /// Storage errors (for async operations)
        invalid-storage-location(string),
        storage-access-denied(string),
    }

    record quota-info {
        used: u32,
        limit: u32,
        reset-time: u64,
        unit: quota-unit,
    }

    enum quota-unit {
        characters,
        requests,
        seconds,
        credits,
    }

    /// Language identification using BCP 47 codes
    type language-code = string;

    /// Voice gender classification
    enum voice-gender {
        male,
        female,
        neutral,
    }

    /// Voice quality tiers
    enum voice-quality {
        standard,
        premium,
        neural,
        studio,
    }

    /// Text input types
    enum text-type {
        plain,
        ssml,
    }

    /// Audio output formats
    enum audio-format {
        mp3,
        wav,
        pcm,
        ogg-opus,
        aac,
        flac,
        mulaw,
        alaw,
    }

    /// Audio quality settings
    record audio-config {
        format: audio-format,
        sample-rate: option<u32>,
        bit-rate: option<u32>,
        channels: option<u8>,
    }

    /// Voice synthesis parameters
    record voice-settings {
        /// Speaking rate (0.25 to 4.0, default 1.0)
        speed: option<f32>,
        /// Pitch adjustment in semitones (-20.0 to 20.0, default 0.0)
        pitch: option<f32>,
        /// Volume gain in dB (-96.0 to 16.0, default 0.0)
        volume: option<f32>,
        /// Voice stability (0.0 to 1.0, provider-specific)
        stability: option<f32>,
        /// Similarity to original (0.0 to 1.0, provider-specific)
        similarity: option<f32>,
        /// Style exaggeration (0.0 to 1.0, provider-specific)
        style: option<f32>,
    }

    /// Audio effects and device optimization
    flags audio-effects {
        telephone-quality,
        headphone-optimized,
        speaker-optimized,
        car-audio-optimized,
        noise-reduction,
        bass-boost,
        treble-boost,
    }

    /// Input text with metadata
    record text-input {
        content: string,
        text-type: text-type,
        language: option<language-code>,
    }

    /// Complete synthesis result
    record synthesis-result {
        audio-data: list<u8>,
        metadata: synthesis-metadata,
    }

    /// Metadata about synthesized audio
    record synthesis-metadata {
        duration-seconds: f32,
        character-count: u32,
        word-count: u32,
        audio-size-bytes: u32,
        request-id: string,
        provider-info: option<string>,
    }

    /// Streaming audio chunk
    record audio-chunk {
        data: list<u8>,
        sequence-number: u32,
        is-final: bool,
        timing-info: option<timing-info>,
    }

    /// Timing and synchronization information
    record timing-info {
        start-time-seconds: f32,
        end-time-seconds: option<f32>,
        text-offset: option<u32>,
        mark-type: option<timing-mark-type>,
    }

    enum timing-mark-type {
        word,
        sentence,
        paragraph,
        ssml-mark,
        viseme,
    }


}

/// Voice discovery and management
interface voices {
    use types.{tts-error, language-code, voice-gender, voice-quality};

    /// Represents a voice that can be used for speech synthesis
    resource voice {
        /// Get voice identification
        get-id: func() -> string;
        get-name: func() -> string;
        get-provider-id: func() -> option<string>;
        
        /// Get voice characteristics
        get-language: func() -> language-code;
        get-additional-languages: func() -> list<language-code>;
        get-gender: func() -> voice-gender;
        get-quality: func() -> voice-quality;
        get-description: func() -> option<string>;
        
        /// Voice capabilities
        supports-ssml: func() -> bool;
        get-sample-rates: func() -> list<u32>;
        get-supported-formats: func() -> list<types.audio-format>;
        
        /// Voice management (may return unsupported-operation)
        update-settings: func(settings: types.voice-settings) -> result<_, tts-error>;
        delete: func() -> result<_, tts-error>;
        clone: func() -> result<voice, tts-error>;
        
        /// Preview voice with sample text
        preview: func(text: string) -> result<list<u8>, tts-error>;
    }

    /// Voice search and filtering
    record voice-filter {
        language: option<language-code>,
        gender: option<voice-gender>,
        quality: option<voice-quality>,
        supports-ssml: option<bool>,
        provider: option<string>,
        search-query: option<string>,
    }

    /// Detailed voice information
    record voice-info {
        id: string,
        name: string,
        language: language-code,
        additional-languages: list<language-code>,
        gender: voice-gender,
        quality: voice-quality,
        description: option<string>,
        provider: string,
        sample-rate: u32,
        is-custom: bool,
        is-cloned: bool,
        preview-url: option<string>,
        use-cases: list<string>,
    }

    /// Resource-based iterator for voice results
    resource voice-results {
        /// Check if more voices are available
        has-more: func() -> bool;
        
        /// Get next batch of voices
        get-next: func() -> result<list<voice-info>, tts-error>;
        
        /// Get total count if available
        get-total-count: func() -> option<u32>;
    }

    /// List available voices with filtering and pagination
    list-voices: func(
        filter: option<voice-filter>
    ) -> result<voice-results, tts-error>;

    /// Get specific voice by ID
    get-voice: func(voice-id: string) -> result<voice, tts-error>;

    /// Search voices by characteristics
    search-voices: func(
        query: string,
        filter: option<voice-filter>
    ) -> result<list<voice-info>, tts-error>;

    /// Get supported languages
    list-languages: func() -> result<list<language-info>, tts-error>;

    record language-info {
        code: language-code,
        name: string,
        native-name: string,
        voice-count: u32,
    }
}

/// Core text-to-speech synthesis operations
interface synthesis {
    use types.{
        text-input, audio-config, voice-settings, audio-effects,
        synthesis-result, tts-error, timing-info
    };
    use voices.{voice};

    /// Synthesis configuration options
    record synthesis-options {
        audio-config: option<audio-config>,
        voice-settings: option<voice-settings>,
        audio-effects: option<audio-effects>,
        enable-timing: option<bool>,
        enable-word-timing: option<bool>,
        seed: option<u32>,
        model-version: option<string>,
        context: option<synthesis-context>,
    }

    /// Context for better synthesis quality
    record synthesis-context {
        previous-text: option<string>,
        next-text: option<string>,
        topic: option<string>,
        emotion: option<string>,
        speaking-style: option<string>,
    }

    /// Convert text to speech (removed async)
    synthesize: func(
        input: text-input,
        voice: borrow<voice>,
        options: option<synthesis-options>
    ) -> result<synthesis-result, tts-error>;

    /// Batch synthesis for multiple inputs (removed async)
    synthesize-batch: func(
        inputs: list<text-input>,
        voice: borrow<voice>,
        options: option<synthesis-options>
    ) -> result<list<synthesis-result>, tts-error>;

    /// Get timing information without audio synthesis
    get-timing-marks: func(
        input: text-input,
        voice: borrow<voice>
    ) -> result<list<timing-info>, tts-error>;

    /// Validate text before synthesis
    validate-input: func(
        input: text-input,
        voice: borrow<voice>
    ) -> result<validation-result, tts-error>;

    record validation-result {
        is-valid: bool,
        character-count: u32,
        estimated-duration: option<f32>,
        warnings: list<string>,
        errors: list<string>,
    }
}

/// Real-time streaming synthesis
interface streaming {
    use types.{
        text-input, audio-config, voice-settings, audio-chunk,
        tts-error, timing-info
    };
    use voices.{voice};
    use synthesis.{synthesis-options};

    /// Streaming synthesis session
    resource synthesis-stream {
        /// Send text for synthesis (can be called multiple times)
        send-text: func(input: text-input) -> result<_, tts-error>;
        
        /// Signal end of input and flush remaining audio
        finish: func() -> result<_, tts-error>;
        
        /// Receive next audio chunk (non-blocking)
        receive-chunk: func() -> result<option<audio-chunk>, tts-error>;
        
        /// Check if more chunks are available
        has-pending-audio: func() -> bool;
        
        /// Get current stream status
        get-status: func() -> stream-status;
        
        /// Close stream and clean up resources
        close: func();
    }

    enum stream-status {
        ready,
        processing,
        finished,
        error,
        closed,
    }

    /// Create streaming synthesis session
    create-stream: func(
        voice: borrow<voice>,
        options: option<synthesis-options>
    ) -> result<synthesis-stream, tts-error>;

    /// Real-time voice conversion streaming
    create-voice-conversion-stream: func(
        target-voice: borrow<voice>,
        options: option<synthesis-options>
    ) -> result<voice-conversion-stream, tts-error>;

    resource voice-conversion-stream {
        /// Send input audio chunks
        send-audio: func(audio-data: list<u8>) -> result<_, tts-error>;
        
        /// Receive converted audio chunks
        receive-converted: func() -> result<option<audio-chunk>, tts-error>;
        
        finish: func() -> result<_, tts-error>;
        close: func();
    }
}

/// Advanced TTS features and voice manipulation
interface advanced {
    use types.{tts-error, audio-config, language-code};
    use voices.{voice};

    /// Voice cloning and creation (removed async)
    create-voice-clone: func(
        name: string,
        audio-samples: list<audio-sample>,
        description: option<string>
    ) -> result<voice, tts-error>;

    record audio-sample {
        data: list<u8>,
        transcript: option<string>,
        quality-rating: option<u8>,
    }

    /// Design synthetic voice (removed async)
    design-voice: func(
        name: string,
        characteristics: voice-design-params
    ) -> result<voice, tts-error>;

    record voice-design-params {
        gender: types.voice-gender,
        age-category: age-category,
        accent: string,
        personality-traits: list<string>,
        reference-voice: option<string>,
    }

    enum age-category {
        child,
        young-adult,
        middle-aged,
        elderly,
    }

    /// Voice-to-voice conversion (removed async)
    convert-voice: func(
        input-audio: list<u8>,
        target-voice: borrow<voice>,
        preserve-timing: option<bool>
    ) -> result<list<u8>, tts-error>;

    /// Generate sound effects from text description (removed async)
    generate-sound-effect: func(
        description: string,
        duration-seconds: option<f32>,
        style-influence: option<f32>
    ) -> result<list<u8>, tts-error>;

    /// Custom pronunciation management
    resource pronunciation-lexicon {
        get-name: func() -> string;
        get-language: func() -> language-code;
        get-entry-count: func() -> u32;
        
        /// Add pronunciation rule
        add-entry: func(word: string, pronunciation: string) -> result<_, tts-error>;
        
        /// Remove pronunciation rule
        remove-entry: func(word: string) -> result<_, tts-error>;
        
        /// Export lexicon content
        export-content: func() -> result<string, tts-error>;
    }

    /// Create custom pronunciation lexicon
    create-lexicon: func(
        name: string,
        language: language-code,
        entries: option<list<pronunciation-entry>>
    ) -> result<pronunciation-lexicon, tts-error>;

    record pronunciation-entry {
        word: string,
        pronunciation: string,
        part-of-speech: option<string>,
    }

    /// Long-form content synthesis with optimization (removed async)
    synthesize-long-form: func(
        content: string,
        voice: borrow<voice>,
        output-location: string,
        chapter-breaks: option<list<u32>>
    ) -> result<long-form-operation, tts-error>;

    resource long-form-operation {
        get-status: func() -> operation-status;
        get-progress: func() -> f32;
        cancel: func() -> result<_, tts-error>;
        get-result: func() -> result<long-form-result, tts-error>;
    }

    enum operation-status {
        pending,
        processing,
        completed,
        failed,
        cancelled,
    }

    record long-form-result {
        output-location: string,
        total-duration: f32,
        chapter-durations: option<list<f32>>,
        metadata: types.synthesis-metadata,
    }
}