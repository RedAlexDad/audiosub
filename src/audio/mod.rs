use anyhow::Result;

pub struct AudioChunk {
    pub data: Vec<f32>,
    pub timestamp_ms: u64,
}

pub trait AudioCapture: Send {
    fn start(&mut self) -> Result<()>;
    fn read(&mut self) -> Result<Option<AudioChunk>>;
    fn stop(&mut self) -> Result<()>;
    fn sample_rate(&self) -> u32;
}

pub mod pulse;
