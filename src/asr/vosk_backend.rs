use anyhow::Result;
use tracing::debug;

use crate::asr::vosk_dl;
use crate::asr::{AsrEngine, Segment};

pub struct VoskEngine {
    vosk: &'static vosk_dl::VoskDl,
    model: Option<vosk_dl::Model>,
    recognizer: Option<vosk_dl::Recognizer>,
    sample_rate: f32,
    segments: Vec<Segment>,
    partial: String,
}

impl VoskEngine {
    pub fn new(sample_rate: f32) -> Result<Self> {
        let vosk = vosk_dl::load()?;
        Ok(Self {
            vosk,
            model: None,
            recognizer: None,
            sample_rate,
            segments: Vec::new(),
            partial: String::new(),
        })
    }

    fn convert_f32_to_i16(audio: &[f32]) -> Vec<i16> {
        audio
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect()
    }
}

impl AsrEngine for VoskEngine {
    fn load_model(&mut self, path: &str) -> Result<()> {
        debug!("Loading Vosk model from: {path}");

        let model = self
            .vosk
            .model_new(path)
            .ok_or_else(|| anyhow::anyhow!("Failed to load Vosk model from {path}"))?;

        let recognizer = self
            .vosk
            .recognizer_new(&model, self.sample_rate)
            .ok_or_else(|| anyhow::anyhow!("Failed to create Vosk recognizer"))?;

        self.vosk.recognizer_set_words(&recognizer, true);

        self.model = Some(model);
        self.recognizer = Some(recognizer);
        self.segments.clear();
        self.partial.clear();

        debug!("Vosk model loaded and recognizer created");
        Ok(())
    }

    fn feed_audio(&mut self, audio: &[f32]) -> Result<()> {
        let pcm = Self::convert_f32_to_i16(audio);

        let rec = self
            .recognizer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Recognizer not initialized"))?;

        let state = self.vosk.accept_waveform(rec, &pcm);
        let json = self.vosk.partial_result(rec);
        let segment = if matches!(state, vosk_dl::DecodingState::Finalized) {
            Self::parse_result(&json)
        } else {
            None
        };

        self.partial = json;

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
        if let Some(rec) = self.recognizer.as_ref() {
            self.partial = self.vosk.partial_result(rec);
        }
        Ok(self.partial.clone())
    }

    fn drain_segments(&mut self) -> Result<Vec<Segment>> {
        Ok(std::mem::take(&mut self.segments))
    }

    fn finalize(&mut self) -> Result<Vec<Segment>> {
        debug!("Finalizing Vosk recognition");

        if let Some(rec) = self.recognizer.as_ref() {
            let json = self.vosk.result(rec);
            if let Some(seg) = Self::parse_result(&json) {
                self.segments.push(seg);
            }
        }

        Ok(std::mem::take(&mut self.segments))
    }

    fn reset(&mut self) -> Result<()> {
        debug!("Resetting Vosk engine");
        if let Some(rec) = self.recognizer.as_ref() {
            self.vosk.recognizer_reset(rec);
        }
        self.segments.clear();
        self.partial.clear();
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct VoskOutput {
    text: String,
    #[serde(default)]
    result: Vec<VoskWord>,
}

#[derive(serde::Deserialize)]
struct VoskWord {
    start: f64,
    end: f64,
}

impl VoskEngine {
    fn parse_result(json: &str) -> Option<Segment> {
        let output: VoskOutput = serde_json::from_str(json).ok()?;
        let text = output.text.trim().to_string();
        if text.is_empty() {
            return None;
        }
        let start_ms = output.result.first().map(|w| (w.start * 1000.0) as u64).unwrap_or(0);
        let end_ms = output.result.last().map(|w| (w.end * 1000.0) as u64).unwrap_or(0);
        Some(Segment { start_ms, end_ms, text })
    }
}
