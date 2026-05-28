use anyhow::Result;
use tracing::info;

pub struct TuiApp;

impl Default for TuiApp {
    fn default() -> Self {
        Self
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<()> {
        info!("TUI mode requested but not yet implemented");
        Ok(())
    }
}
