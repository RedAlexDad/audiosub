use anyhow::Result;

pub struct VoskEngine;

impl Default for VoskEngine {
    fn default() -> Self {
        Self
    }
}

impl VoskEngine {
    pub fn new() -> Self {
        Self
    }
}

impl super::AsrEngine for VoskEngine {
    fn load_model(&mut self, _path: &str) -> Result<()> {
        Ok(())
    }

    fn transcribe(&mut self, _audio: &[f32]) -> Result<Vec<super::Segment>> {
        Ok(vec![])
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}
