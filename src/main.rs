use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

mod audio;
use crate::audio::AudioCapture;
mod asr;
use crate::asr::AsrEngine;
mod cli;
mod config;
mod logging;
mod subtitle;
use crate::subtitle::{SubtitleBuffer, SubtitleOutput};
mod tui;

fn default_model_path() -> PathBuf {
    let cache_dir = directories::BaseDirs::new()
        .map(|d| d.cache_dir().join("audiosub").join("models"))
        .unwrap_or_else(|| PathBuf::from("~/.cache/audiosub/models"));
    cache_dir.join("vosk-model-small-en-us-0.15")
}

#[cfg(feature = "vosk")]
fn create_engine(sample_rate: f32) -> Box<dyn AsrEngine> {
    Box::new(asr::vosk_backend::VoskEngine::new(sample_rate))
}

#[cfg(not(any(feature = "vosk", feature = "whisper")))]
fn create_engine(_sample_rate: f32) -> Box<dyn AsrEngine> {
    panic!("No ASR engine enabled. Enable 'vosk' or 'whisper' feature.");
}

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

    let source_rate = cfg.audio.sample_rate;

    let mut capture = audio::PulseCapture::new(&device, source_rate);
    capture.start()?;

    let engine_rate = capture.sample_rate();

    tracing::info!("Capturing from: {} ({} → {} Hz)", device, source_rate, engine_rate);

    let model_path = args.model.clone().unwrap_or_else(default_model_path);

    let mut engine = create_engine(engine_rate as f32);
    engine.load_model(model_path.to_str().unwrap_or(""))?;
    tracing::info!("ASR engine loaded model from: {}", model_path.display());

    let output_path = args
        .output
        .clone()
        .or_else(|| Some(cfg.subtitle.output.clone()))
        .unwrap_or_else(|| PathBuf::from("output.srt"));
    let output_format = args
        .format
        .clone()
        .unwrap_or(cfg.subtitle.format.clone());

    let mut output = SubtitleOutput::create(&output_path, &output_format)?;
    let mut buffer = SubtitleBuffer::new(cfg.subtitle.buffer_ms);

    let chunk_size = (source_rate as usize) / 10; // 100ms chunks of source audio
    let duration = Duration::from_secs(30);

    let start = std::time::Instant::now();
    let mut total_samples = 0usize;
    let mut segment_count = 0usize;

    while start.elapsed() < duration {
        if let Some(chunk) = capture.read(chunk_size)? {
            total_samples += chunk.data.len();

            engine.feed_audio(&chunk.data)?;

            let partial = engine.partial_text()?;
            if !partial.is_empty() {
                tracing::debug!("Partial: {}", partial);
            }

            let stream_pos_ms =
                (total_samples as u64 * 1000) / engine_rate as u64;

            for seg in engine.drain_segments()? {
                segment_count += 1;
                tracing::info!(
                    "[{}] {:06}:{:06} --> {:06}:{:06}  {}",
                    segment_count,
                    seg.start_ms / 60000,
                    seg.start_ms % 60000 / 1000,
                    seg.end_ms / 60000,
                    seg.end_ms % 60000 / 1000,
                    seg.text
                );
                buffer.push(seg.clone());
            }

            for ready in buffer.flush(stream_pos_ms) {
                output.append(&ready)?;
            }
        }
    }

    capture.stop()?;

    let final_segments = engine.finalize()?;
    for seg in &final_segments {
        segment_count += 1;
        tracing::info!(
            "[{}] {:06}:{:06} --> {:06}:{:06}  {}",
            segment_count,
            seg.start_ms / 60000,
            seg.start_ms % 60000 / 1000,
            seg.end_ms / 60000,
            seg.end_ms % 60000 / 1000,
            seg.text
        );
        buffer.push(seg.clone());
    }

    for ready in buffer.drain() {
        output.append(&ready)?;
    }

    output.close()?;

    tracing::info!(
        "Session complete: {} samples ({:.1}s) in {:.1}s, {} segments, output: {:?}",
        total_samples,
        total_samples as f64 / engine_rate as f64,
        start.elapsed().as_secs_f64(),
        segment_count,
        output_path,
    );

    Ok(())
}
