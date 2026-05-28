use anyhow::Result;
use tracing_subscriber::EnvFilter;

pub fn init(verbose: u8) -> Result<()> {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(format!("audiosub={}", level)));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    Ok(())
}
