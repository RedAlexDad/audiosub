use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

pub fn init(verbose: u8, use_file_log: bool) -> Result<()> {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("audiosub={},whisper_rs=warn,ggml=warn", level)));

    if use_file_log {
        let log_file = std::fs::File::create("/tmp/audiosub_stderr.log")
            .context("failed to create TUI log file /tmp/audiosub_stderr.log")?;

        let panic_file = log_file
            .try_clone()
            .context("failed to clone log file for panic hook")?;
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            use std::io::Write;
            let _ = writeln!(&panic_file, "Panic: {info}");
            prev(info);
        }));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(move || {
                log_file
                    .try_clone()
                    .expect("failed to clone log file for tracing writer")
            })
            .with_target(true)
            .with_thread_ids(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    }

    Ok(())
}
