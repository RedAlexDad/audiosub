use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub trait AsrEngine: Send {
    fn load_model(&mut self, path: &str) -> Result<()>;
    fn feed_audio(&mut self, audio: &[f32]) -> Result<()>;
    fn partial_text(&mut self) -> Result<String>;
    fn drain_segments(&mut self) -> Result<Vec<Segment>>;
    fn finalize(&mut self) -> Result<Vec<Segment>>;
    fn reset(&mut self) -> Result<()>;
}

#[cfg(feature = "vosk")]
pub mod vosk_backend;
#[cfg(feature = "whisper")]
pub mod whisper_backend;
