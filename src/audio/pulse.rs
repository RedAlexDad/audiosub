use anyhow::Result;

pub struct PulseCapture;

impl Default for PulseCapture {
    fn default() -> Self {
        Self
    }
}

impl PulseCapture {
    pub fn new(_device: &str) -> Self {
        Self
    }
}

impl super::AudioCapture for PulseCapture {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn read(&mut self) -> Result<Option<super::AudioChunk>> {
        Ok(None)
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        16000
    }
}
