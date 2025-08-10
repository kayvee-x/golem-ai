use golem_tts::export_tts;

mod advanced;
mod client;
mod streaming;
mod synthesis;
mod voices;

pub struct DeepgramComponent;
export_tts!(DeepgramComponent with_types_in golem_tts);
