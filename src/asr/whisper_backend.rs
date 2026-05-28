use anyhow::Result;

pub struct WhisperEngine;

impl Default for WhisperEngine {
    fn default() -> Self {
        Self
    }
}

impl WhisperEngine {
    pub fn new() -> Self {
        Self
    }
}

impl super::AsrEngine for WhisperEngine {
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
