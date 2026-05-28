use anyhow::Result;
use std::time::Instant;

mod monitor;
mod pulse;
mod resampler;

pub use monitor::{find_default_monitor, list_sources};
pub use pulse::PulseCapture;
pub use resampler::AudioResampler;

pub struct AudioChunk {
    pub data: Vec<f32>,
    pub timestamp: Instant,
    pub sample_rate: u32,
}

pub trait AudioCapture: Send {
    fn start(&mut self) -> Result<()>;
    fn read(&mut self, chunk_size: usize) -> Result<Option<AudioChunk>>;
    fn stop(&mut self) -> Result<()>;
    fn sample_rate(&self) -> u32;
}
