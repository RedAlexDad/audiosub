use anyhow::Result;

use crate::asr::{AsrEngine, Segment};

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

impl AsrEngine for WhisperEngine {
    fn load_model(&mut self, _path: &str) -> Result<()> {
        Ok(())
    }

    fn feed_audio(&mut self, _audio: &[f32]) -> Result<()> {
        Ok(())
    }

    fn partial_text(&mut self) -> Result<String> {
        Ok(String::new())
    }

    fn drain_segments(&mut self) -> Result<Vec<Segment>> {
        Ok(vec![])
    }

    fn finalize(&mut self) -> Result<Vec<Segment>> {
        Ok(vec![])
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}
