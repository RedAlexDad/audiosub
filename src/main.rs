#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use std::time::Duration;

mod audio;
use crate::audio::AudioCapture;
mod asr;
mod cli;
mod config;
mod logging;
mod subtitle;
mod tui;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    logging::init(args.verbose)?;

    let cfg = config::Config::load(args.config.as_ref())?;

    tracing::info!("audiosub v{} starting", env!("CARGO_PKG_VERSION"));
    tracing::debug!("Config: {:?}", cfg);

    if args.tui {
        let mut app = tui::TuiApp::new();
        return app.run();
    }

    let device = args
        .device
        .clone()
        .or_else(|| {
            if cfg.audio.device == "default" {
                audio::find_default_monitor().ok()
            } else {
                Some(cfg.audio.device.clone())
            }
        })
        .unwrap_or_else(|| "default".into());

    let mut capture = audio::PulseCapture::new(&device, cfg.audio.sample_rate);
    capture.start()?;

    tracing::info!("Capturing from: {}", device);

    let chunk_size = (cfg.audio.sample_rate as usize) / 10; // 100ms chunks
    let duration = Duration::from_secs(5);

    let start = std::time::Instant::now();
    let mut total_samples = 0usize;

    while start.elapsed() < duration {
        if let Some(chunk) = capture.read(chunk_size)? {
            total_samples += chunk.data.len();
            tracing::info!(
                "chunk: {} samples, {:.1}s of audio, peak: {:.2}",
                chunk.data.len(),
                chunk.data.len() as f64 / chunk.sample_rate as f64,
                chunk.data.iter().map(|&s| s.abs()).fold(0.0f32, f32::max)
            );
        }
    }

    capture.stop()?;
    tracing::info!(
        "Captured {} samples ({:.1}s) in {:.1}s",
        total_samples,
        total_samples as f64 / cfg.audio.sample_rate as f64,
        start.elapsed().as_secs_f64()
    );

    Ok(())
}
