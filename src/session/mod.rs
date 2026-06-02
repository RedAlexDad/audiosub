pub mod model;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;

use crate::asr::AsrEngine;
use crate::audio::{AudioCapture, PulseCapture};
use crate::cli::Cli;
use crate::config::Config;
use crate::subtitle::{SubtitleBuffer, SubtitleOutput};

use self::model::resolve_model_path;

pub fn run_session(
    args: &Cli,
    cfg: &Config,
    device: &str,
    source_rate: u32,
    duration: Duration,
    engine_name: &str,
) -> Result<()> {
    let mut capture = PulseCapture::new(device, source_rate);
    capture.start()?;

    let engine_rate = capture.sample_rate();

    tracing::info!("Capturing from: {device} ({source_rate} → {engine_rate} Hz)");

    let effective = resolve_engine(engine_name);
    let model_path = resolve_model_path(&effective, args.model.as_ref(), &cfg.asr);
    if model_path.is_empty() {
        anyhow::bail!(
            "No model found. Place a .gguf or .bin model next to the binary, \
             use --model <path>, or run `audiosub --download-model`"
        );
    }
    let mut engine = create_engine(&effective, engine_rate as f32)?;
    engine.load_model(&model_path)?;
    tracing::info!(
        "ASR engine '{engine}' loaded model from: {model_path}",
        engine = effective
    );

    let output_path = args
        .output
        .clone()
        .or_else(|| Some(cfg.subtitle.output.clone()))
        .unwrap_or_else(|| PathBuf::from("output.srt"));
    let output_format = args.format.clone().unwrap_or(cfg.subtitle.format.clone());

    let max_duration = args.max_duration.unwrap_or(cfg.subtitle.max_duration_ms);
    let mut output = SubtitleOutput::create(&output_path, &output_format)?;
    let mut buffer = SubtitleBuffer::new(cfg.subtitle.buffer_ms, max_duration);

    let chunk_size = (source_rate as usize) / 10;
    let start = std::time::Instant::now();
    let mut total_samples = 0usize;
    let mut segment_count = 0usize;

    while start.elapsed() < duration {
        if let Some(chunk) = capture.read(chunk_size)? {
            total_samples += chunk.data.len();

            engine.feed_audio(&chunk.data)?;

            let partial = engine.partial_text()?;
            if !partial.is_empty() {
                tracing::debug!("Partial: {partial}");
            }

            let stream_pos_ms = (total_samples as u64 * 1000) / engine_rate as u64;

            for seg in engine.drain_segments()? {
                for split in crate::subtitle::split_segment(seg, max_duration) {
                    segment_count += 1;
                    tracing::info!(
                        "[{segment_count}] {:06}:{:06} --> {:06}:{:06}  {}",
                        split.start_ms / 60000,
                        split.start_ms % 60000 / 1000,
                        split.end_ms / 60000,
                        split.end_ms % 60000 / 1000,
                        split.text
                    );
                    buffer.push(split);
                }
            }

            for ready in buffer.flush(stream_pos_ms) {
                output.append(&ready)?;
            }
        }
    }

    capture.stop()?;

    for seg in engine.finalize()? {
        for split in crate::subtitle::split_segment(seg, max_duration) {
            segment_count += 1;
            tracing::info!(
                "[{segment_count}] {:06}:{:06} --> {:06}:{:06}  {}",
                split.start_ms / 60000,
                split.start_ms % 60000 / 1000,
                split.end_ms / 60000,
                split.end_ms % 60000 / 1000,
                split.text
            );
            buffer.push(split);
        }
    }

    for ready in buffer.drain() {
        output.append(&ready)?;
    }

    output.close()?;

    tracing::info!(
        "Session complete: {total_samples} samples ({:.1}s) in {:.1}s, {segment_count} segments, output: {output_path:?}",
        total_samples as f64 / engine_rate as f64,
        start.elapsed().as_secs_f64(),
    );

    Ok(())
}

pub fn resolve_engine(engine_name: &str) -> String {
    match engine_name {
        "vosk" if crate::asr::vosk_dl::is_available() => "vosk".into(),
        "whisper" => "whisper".into(),
        _ => {
            let detected = crate::session::model::detect_engine();
            if detected != "whisper" || cfg!(feature = "whisper") {
                tracing::warn!("Unknown ASR engine '{engine_name}', falling back to {detected}");
                detected.into()
            } else {
                panic!("No ASR backend available (install libvosk.so or enable whisper feature)");
            }
        }
    }
}

pub fn create_engine(engine_name: &str, sample_rate: f32) -> Result<Box<dyn AsrEngine>> {
    match engine_name {
        "vosk" => Ok(Box::new(crate::asr::vosk_backend::VoskEngine::new(sample_rate)?)),
        #[cfg(feature = "whisper")]
        "whisper" => Ok(Box::new(crate::asr::whisper_backend::WhisperEngine::new(sample_rate))),
        #[cfg(not(feature = "whisper"))]
        "whisper" => anyhow::bail!("Whisper backend not compiled (enable 'whisper' feature)"),
        _ => anyhow::bail!("Unknown ASR engine '{engine_name}'"),
    }
}
