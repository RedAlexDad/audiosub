use std::sync::mpsc;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::{AudioCapture, AudioChunk, AudioResampler};

pub struct CpalCapture {
    device_name: String,
    source_rate: u32,
    target_rate: u32,
    stream: Option<cpal::Stream>,
    rx: Option<mpsc::Receiver<Vec<f32>>>,
    resampler: Option<AudioResampler>,
    buffer: Vec<f32>,
}

impl CpalCapture {
    pub fn new(device: &str, source_rate: u32) -> Self {
        Self {
            device_name: device.to_string(),
            source_rate,
            target_rate: 16000,
            stream: None,
            rx: None,
            resampler: None,
            buffer: Vec::new(),
        }
    }

    pub fn read_raw(&mut self, n: usize) -> Result<Option<Vec<f32>>> {
        let rx = self.rx.as_ref().context("CPAL not started")?;
        let mut samples = Vec::with_capacity(n);
        while samples.len() < n {
            if let Ok(chunk) = rx.try_recv() {
                samples.extend(chunk);
            } else {
                break;
            }
        }
        if samples.is_empty() {
            return Ok(None);
        }
        self.buffer.extend(samples);
        let available = self.buffer.len().min(n);
        let result: Vec<f32> = self.buffer.drain(..available).collect();
        Ok(Some(result))
    }
}

impl AudioCapture for CpalCapture {
    fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();

        let device = if self.device_name.is_empty() || self.device_name == "default" {
            host.default_input_device().context("No default input device found")?
        } else {
            host.input_devices()?
                .find(|d| d.description().ok().is_some_and(|desc| desc.name() == self.device_name))
                .context(format!("Device '{}' not found", self.device_name))?
        };

        let config = device.default_input_config()?;
        let source_rate = config.sample_rate();
        self.source_rate = source_rate;

        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        self.rx = Some(rx);

        let err_fn = move |err| {
            tracing::error!("CPAL stream error: {err}");
        };

        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    let _ = tx.send(data.to_vec());
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let float_data: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let _ = tx.send(float_data);
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    let float_data: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();
                    let _ = tx.send(float_data);
                },
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;

        let resampler = AudioResampler::new(source_rate, self.target_rate).context("Failed to create resampler")?;

        tracing::info!(
            "CPAL capture started: device={}, rate={}, format={:?}",
            device.description()?.name(),
            source_rate,
            sample_format,
        );

        self.stream = Some(stream);
        self.resampler = Some(resampler);
        Ok(())
    }

    fn read(&mut self, chunk_size: usize) -> Result<Option<AudioChunk>> {
        let rx = self.rx.as_ref().context("CPAL not started")?;
        let resampler = self.resampler.as_mut().context("Resampler not initialized")?;

        while self.buffer.len() < chunk_size {
            match rx.try_recv() {
                Ok(chunk) => self.buffer.extend(chunk),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(None),
            }
        }

        if self.buffer.is_empty() {
            return Ok(None);
        }

        let data: Vec<f32> = self.buffer.drain(..).collect();
        let resampled = resampler.process(&data)?;

        if resampled.is_empty() {
            return Ok(None);
        }

        Ok(Some(AudioChunk { data: resampled }))
    }

    fn stop(&mut self) -> Result<()> {
        self.stream.take();
        self.resampler.take();
        self.rx.take();
        self.buffer.clear();
        tracing::info!("CPAL capture stopped");
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.target_rate
    }
}

#[allow(dead_code)]
pub fn list_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices: Vec<String> = host
        .input_devices()?
        .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
        .collect();
    Ok(devices)
}
