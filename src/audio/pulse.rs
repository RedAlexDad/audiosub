use anyhow::{Context, Result};
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use std::time::Instant;

use super::{AudioCapture, AudioChunk, AudioResampler};

pub struct PulseCapture {
    device: String,
    source_rate: u32,
    target_rate: u32,
    resampler: Option<AudioResampler>,
    pa: Option<Simple>,
}

impl PulseCapture {
    pub fn new(device: &str, source_rate: u32) -> Self {
        Self {
            device: device.to_string(),
            source_rate,
            target_rate: 16000,
            resampler: None,
            pa: None,
        }
    }

    pub fn with_target_rate(mut self, target_rate: u32) -> Self {
        self.target_rate = target_rate;
        self
    }
}

impl AudioCapture for PulseCapture {
    fn start(&mut self) -> Result<()> {
        let spec = Spec {
            format: Format::FLOAT32NE,
            channels: 1,
            rate: self.source_rate,
        };

        if !spec.is_valid() {
            anyhow::bail!("Invalid PulseAudio sample spec: rate={}, channels=1", self.source_rate);
        }

        let pa = Simple::new(
            None,
            "audiosub",
            Direction::Record,
            Some(&self.device),
            "audiosub-capture",
            &spec,
            None,
            None,
        )
        .context(format!("Failed to connect to PulseAudio source '{}'", self.device))?;

        let resampler =
            AudioResampler::new(self.source_rate, self.target_rate).context("Failed to create audio resampler")?;

        tracing::info!(
            "PulseAudio capture started: device={}, source_rate={}, target_rate={}",
            self.device,
            self.source_rate,
            self.target_rate
        );
        self.pa = Some(pa);
        self.resampler = Some(resampler);
        Ok(())
    }

    fn read(&mut self, chunk_size: usize) -> Result<Option<AudioChunk>> {
        let pa = self.pa.as_ref().context("PulseAudio not started")?;
        let resampler = self.resampler.as_mut().context("Resampler not initialized")?;

        let byte_len = chunk_size * 4; // f32 = 4 bytes
        let mut buf = vec![0u8; byte_len];

        if let Err(e) = pa.read(&mut buf) {
            tracing::warn!("PulseAudio read error: {}", e);
            return Ok(None);
        }

        let timestamp = Instant::now();

        let data: Vec<f32> = buf
            .chunks_exact(4)
            .map(|b| f32::from_ne_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let resampled = resampler.process(&data)?;

        if resampled.is_empty() {
            return Ok(None);
        }

        Ok(Some(AudioChunk {
            data: resampled,
            timestamp,
            sample_rate: self.target_rate,
        }))
    }

    fn stop(&mut self) -> Result<()> {
        self.pa.take();
        self.resampler.take();
        tracing::info!("PulseAudio capture stopped");
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.target_rate
    }
}
