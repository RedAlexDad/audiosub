#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;

mod asr;
mod audio;
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
        app.run()?;
    } else {
        tracing::info!("CLI mode — run with --tui for interactive interface");
    }

    Ok(())
}
