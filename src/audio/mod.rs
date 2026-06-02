use anyhow::Result;

mod cpal;
mod monitor;
mod pulse;
mod resampler;

pub use cpal::CpalCapture;
pub use monitor::{find_default_monitor, list_sources};
pub use pulse::PulseCapture;
pub use resampler::AudioResampler;

#[cfg(target_os = "linux")]
pub use pulse::PulseCapture as DefaultCapture;

#[cfg(not(target_os = "linux"))]
pub use cpal::CpalCapture as DefaultCapture;

#[cfg(not(target_os = "linux"))]
pub use cpal::list_devices;

pub struct AudioChunk {
    pub data: Vec<f32>,
}

pub trait AudioCapture: Send {
    fn start(&mut self) -> Result<()>;
    fn read(&mut self, chunk_size: usize) -> Result<Option<AudioChunk>>;
    fn stop(&mut self) -> Result<()>;
    fn sample_rate(&self) -> u32;
}
