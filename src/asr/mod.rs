use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub trait AsrEngine: Send {
    fn load_model(&mut self, path: &str) -> Result<()>;
    fn transcribe(&mut self, audio: &[f32]) -> Result<Vec<Segment>>;
    fn reset(&mut self) -> Result<()>;
}

pub mod vosk_backend;
pub mod whisper_backend;
