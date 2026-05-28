use anyhow::{Context, Result};
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use std::time::Instant;

use super::{AudioCapture, AudioChunk};

pub struct PulseCapture {
    device: String,
    sample_rate: u32,
    pa: Option<Simple>,
}

impl PulseCapture {
    pub fn new(device: &str, sample_rate: u32) -> Self {
        Self {
            device: device.to_string(),
            sample_rate,
            pa: None,
        }
    }
}

impl AudioCapture for PulseCapture {
    fn start(&mut self) -> Result<()> {
        let spec = Spec {
            format: Format::FLOAT32NE,
            channels: 1,
            rate: self.sample_rate,
        };

        if !spec.is_valid() {
            anyhow::bail!("Invalid PulseAudio sample spec: rate={}, channels=1", self.sample_rate);
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

        tracing::info!(
            "PulseAudio capture started: device={}, rate={}",
            self.device,
            self.sample_rate
        );
        self.pa = Some(pa);
        Ok(())
    }

    fn read(&mut self, chunk_size: usize) -> Result<Option<AudioChunk>> {
        let pa = self.pa.as_ref().context("PulseAudio not started")?;

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

        Ok(Some(AudioChunk {
            data,
            timestamp,
            sample_rate: self.sample_rate,
        }))
    }

    fn stop(&mut self) -> Result<()> {
        self.pa.take();
        tracing::info!("PulseAudio capture stopped");
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
