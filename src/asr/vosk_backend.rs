use anyhow::Result;
use tracing::debug;

use crate::asr::{AsrEngine, Segment};

pub struct VoskEngine {
    model: Option<vosk::Model>,
    recognizer: Option<vosk::Recognizer>,
    sample_rate: f32,
    segments: Vec<Segment>,
    partial: String,
}

impl VoskEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            model: None,
            recognizer: None,
            sample_rate,
            segments: Vec::new(),
            partial: String::new(),
        }
    }

    fn recognizer_mut(&mut self) -> Result<&mut vosk::Recognizer> {
        self.recognizer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Recognizer not initialized, call load_model first"))
    }

    fn convert_f32_to_i16(audio: &[f32]) -> Vec<i16> {
        audio
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect()
    }

    fn extract_segments_from_result(recognizer: &mut vosk::Recognizer) -> Option<Segment> {
        let result = recognizer.result();
        let single = result.single()?;
        let text = single.text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        let start_ms = single.result.first().map(|w| (w.start * 1000.0) as u64).unwrap_or(0);

        let end_ms = single.result.last().map(|w| (w.end * 1000.0) as u64).unwrap_or(0);

        Some(Segment { start_ms, end_ms, text })
    }
}

impl AsrEngine for VoskEngine {
    fn load_model(&mut self, path: &str) -> Result<()> {
        debug!("Loading Vosk model from: {path}");

        let model = vosk::Model::new(path).ok_or_else(|| anyhow::anyhow!("Failed to load Vosk model from {path}"))?;

        let mut recognizer = vosk::Recognizer::new(&model, self.sample_rate)
            .ok_or_else(|| anyhow::anyhow!("Failed to create Vosk recognizer"))?;

        recognizer.set_words(true);

        self.model = Some(model);
        self.recognizer = Some(recognizer);
        self.segments.clear();
        self.partial.clear();

        debug!("Vosk model loaded and recognizer created");
        Ok(())
    }

    fn feed_audio(&mut self, audio: &[f32]) -> Result<()> {
        let pcm = Self::convert_f32_to_i16(audio);

        let (state_text, segment) = {
            let recognizer = self.recognizer_mut()?;

            let state = recognizer.accept_waveform(&pcm)?;
            let partial = recognizer.partial_result();
            let state_text = partial.partial.to_string();
            let segment = if state == vosk::DecodingState::Finalized {
                Self::extract_segments_from_result(recognizer)
            } else {
                None
            };

            (state_text, segment)
        };

        self.partial = state_text;

        if let Some(seg) = segment {
            debug!(
                "Segment finalized: \"{}\" ({}ms-{}ms)",
                seg.text, seg.start_ms, seg.end_ms
            );
            self.segments.push(seg);
        }

        Ok(())
    }

    fn partial_text(&mut self) -> Result<String> {
        if let Some(recognizer) = self.recognizer.as_mut() {
            let partial = recognizer.partial_result();
            self.partial = partial.partial.to_string();
        }
        Ok(self.partial.clone())
    }

    fn drain_segments(&mut self) -> Result<Vec<Segment>> {
        Ok(std::mem::take(&mut self.segments))
    }

    fn finalize(&mut self) -> Result<Vec<Segment>> {
        debug!("Finalizing Vosk recognition");

        let final_segment = self.recognizer.as_mut().and_then(Self::extract_segments_from_result);

        if let Some(seg) = final_segment {
            debug!("Final segment: \"{}\"", seg.text);
            self.segments.push(seg);
        }

        Ok(std::mem::take(&mut self.segments))
    }

    fn reset(&mut self) -> Result<()> {
        debug!("Resetting Vosk engine");
        if let Some(recognizer) = self.recognizer.as_mut() {
            recognizer.reset();
        }
        self.segments.clear();
        self.partial.clear();
        Ok(())
    }
}
