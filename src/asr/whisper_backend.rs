use anyhow::{Context, Result};
use tracing::debug;

use crate::asr::{AsrEngine, Segment};

pub struct WhisperEngine {
    ctx: Option<whisper_rs::WhisperContext>,
    sample_rate: f32,
    audio_buffer: Vec<f32>,
    total_samples: u64,
    buffer_start_sample: u64,
    consumed_ms: u64,
    segments: Vec<Segment>,
    partial: String,
    samples_since_last_run: usize,
    update_interval_samples: usize,
    min_samples: usize,
    max_buffer_samples: usize,
}

impl WhisperEngine {
    pub fn new(sample_rate: f32) -> Self {
        let update_ms = 3000u64;
        Self {
            ctx: None,
            sample_rate,
            audio_buffer: Vec::new(),
            total_samples: 0,
            buffer_start_sample: 0,
            consumed_ms: 0,
            segments: Vec::new(),
            partial: String::new(),
            samples_since_last_run: 0,
            update_interval_samples: (update_ms as f32 / 1000.0 * sample_rate) as usize,
            min_samples: (sample_rate as usize) * 2,
            max_buffer_samples: (sample_rate as usize) * 30,
        }
    }

    fn run_inference(&mut self) -> Result<()> {
        let ctx = self.ctx.as_ref().context("Whisper context not initialized")?;
        let mut state = ctx.create_state()?;

        let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_translate(false);
        params.set_language(Some("ru"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_no_timestamps(false);
        params.set_single_segment(false);
        params.set_max_len(100);

        state.full(params, &self.audio_buffer)?;

        let n_segments = state.full_n_segments()?;
        let buffer_start_ms = self.buffer_start_sample as f64 / self.sample_rate as f64 * 1000.0;

        for i in 0..n_segments {
            let text = state.full_get_segment_text(i)?;
            if text.trim().is_empty() {
                continue;
            }

            let t0_ns = state.full_get_segment_t0(i)?;
            let t1_ns = state.full_get_segment_t1(i)?;

            let abs_start_ms = buffer_start_ms + t0_ns as f64 / 1_000_000.0;
            let abs_end_ms = buffer_start_ms + t1_ns as f64 / 1_000_000.0;

            if abs_end_ms <= self.consumed_ms as f64 {
                continue;
            }

            let start_ms = abs_start_ms as u64;
            let end_ms = abs_end_ms as u64;

            self.segments.push(Segment {
                start_ms,
                end_ms,
                text: text.clone(),
            });

            if end_ms > self.consumed_ms {
                self.consumed_ms = end_ms;
            }

            self.partial = text;
        }
        self.consumed_ms = self
            .consumed_ms
            .max((self.buffer_start_sample as f64 / self.sample_rate as f64 * 1000.0) as u64);

        let max_consumed_sample = (self.consumed_ms as f64 / 1000.0 * self.sample_rate as f64) as u64;
        if max_consumed_sample > self.buffer_start_sample {
            let drop_samples = (max_consumed_sample - self.buffer_start_sample) as usize;
            let drop_samples = drop_samples.min(self.audio_buffer.len() / 2);
            if drop_samples > 0 {
                self.audio_buffer.drain(..drop_samples);
                self.buffer_start_sample += drop_samples as u64;
            }
        }

        if self.audio_buffer.len() > self.max_buffer_samples {
            let excess = self.audio_buffer.len() - self.max_buffer_samples;
            self.audio_buffer.drain(..excess);
            self.buffer_start_sample += excess as u64;
        }

        self.samples_since_last_run = 0;
        debug!(
            "Whisper inference: {} segments, buffer {} samples, consumed_ms={}",
            n_segments,
            self.audio_buffer.len(),
            self.consumed_ms
        );

        Ok(())
    }
}

impl AsrEngine for WhisperEngine {
    fn load_model(&mut self, path: &str) -> Result<()> {
        debug!("Loading Whisper model from: {path}");

        let ctx = whisper_rs::WhisperContext::new_with_params(path, whisper_rs::WhisperContextParameters::default())?;

        self.ctx = Some(ctx);
        self.audio_buffer.clear();
        self.total_samples = 0;
        self.buffer_start_sample = 0;
        self.consumed_ms = 0;
        self.segments.clear();
        self.partial.clear();
        self.samples_since_last_run = 0;

        debug!("Whisper model loaded");
        Ok(())
    }

    fn feed_audio(&mut self, audio: &[f32]) -> Result<()> {
        self.audio_buffer.extend_from_slice(audio);
        self.total_samples += audio.len() as u64;
        self.samples_since_last_run += audio.len();

        if self.ctx.is_none() {
            return Ok(());
        }

        if self.total_samples < self.min_samples as u64 {
            return Ok(());
        }

        if self.samples_since_last_run >= self.update_interval_samples {
            self.run_inference()?;
        }

        Ok(())
    }

    fn partial_text(&mut self) -> Result<String> {
        Ok(self.partial.clone())
    }

    fn drain_segments(&mut self) -> Result<Vec<Segment>> {
        Ok(std::mem::take(&mut self.segments))
    }

    fn finalize(&mut self) -> Result<Vec<Segment>> {
        debug!("Finalizing Whisper recognition");

        if !self.audio_buffer.is_empty() && self.ctx.is_some() {
            self.run_inference()?;
        }

        Ok(std::mem::take(&mut self.segments))
    }

    fn reset(&mut self) -> Result<()> {
        debug!("Resetting Whisper engine");
        self.audio_buffer.clear();
        self.total_samples = 0;
        self.buffer_start_sample = 0;
        self.consumed_ms = 0;
        self.segments.clear();
        self.partial.clear();
        self.samples_since_last_run = 0;
        Ok(())
    }
}
