use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;

use audiosub::audio::AudioCapture;
use audiosub::cli::Cli;
use audiosub::session;
use audiosub::subtitle::{SubtitleBuffer, SubtitleOutput};

fn main() -> Result<()> {
    let args = Cli::parse();

    let use_file_log = !args.no_tui && cfg!(feature = "tui");
    audiosub::logging::init(args.verbose, use_file_log)?;

    #[cfg(feature = "whisper")]
    whisper_rs::install_logging_hooks();

    let cfg = audiosub::config::Config::load(args.config.as_ref())?;

    let engine_name = args.engine.as_deref().unwrap_or(&cfg.asr.engine);
    tracing::info!("ASR engine: {engine_name}");

    tracing::info!("audiosub v{} starting", env!("CARGO_PKG_VERSION"));
    tracing::debug!("Config: {:?}", cfg);

    if args.list_devices {
        let sources = audiosub::audio::list_sources()?;
        println!("Available PulseAudio monitor sources:");
        for s in &sources {
            println!("  {s}");
        }
        return Ok(());
    }

    let device = args
        .device
        .clone()
        .or_else(|| {
            if cfg.audio.device == "default" {
                audiosub::audio::find_default_monitor().ok()
            } else {
                Some(cfg.audio.device.clone())
            }
        })
        .unwrap_or_else(|| "default".into());

    let duration = Duration::from_secs(args.duration.unwrap_or(u64::MAX));
    let max_duration = args.max_duration.unwrap_or(cfg.subtitle.max_duration_ms);

    if args.no_tui {
        return session::run_session(&args, &cfg, &device, cfg.audio.sample_rate, duration, engine_name);
    }

    #[cfg(feature = "tui")]
    {
        let mut capture = audiosub::audio::PulseCapture::new(&device, cfg.audio.sample_rate);
        capture.start()?;

        let model_path = session::model::resolve_model_path(args.model.as_ref(), &cfg.asr.model_path);
        let mut engine = session::create_engine(engine_name, 16000.0);
        engine.load_model(&model_path)?;
        tracing::info!(
            "ASR engine '{engine}' loaded model from: {model_path}",
            engine = engine_name
        );

        let output_path = args
            .output
            .clone()
            .or_else(|| Some(cfg.subtitle.output.clone()))
            .unwrap_or_else(|| PathBuf::from("output.srt"));
        let output_format = args.format.clone().unwrap_or(cfg.subtitle.format.clone());
        let output = SubtitleOutput::create(&output_path, &output_format)?;
        let buffer = SubtitleBuffer::new(cfg.subtitle.buffer_ms, max_duration);

        let chunk_size = (cfg.audio.sample_rate as usize) / 10;
        audiosub::tui::worker::run_tui(
            capture,
            engine,
            output,
            buffer,
            cfg.audio.sample_rate,
            chunk_size,
            max_duration,
        )
    }

    #[cfg(not(feature = "tui"))]
    {
        anyhow::bail!("TUI mode requires the 'tui' feature: cargo build --features tui")
    }
}
