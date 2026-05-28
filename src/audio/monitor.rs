use anyhow::{Context, Result};
use std::process::Command;

pub fn find_default_monitor() -> Result<String> {
    let pactl_out = Command::new("pactl")
        .args(["info"])
        .output()
        .context("Failed to run pactl info")?;

    let stdout = String::from_utf8_lossy(&pactl_out.stdout);
    let default_sink = stdout
        .lines()
        .find_map(|l| l.strip_prefix("Default Sink:").map(|s| s.trim()))
        .context("Could not find Default Sink in pactl info")?;

    let monitor_name = format!("{}.monitor", default_sink);
    tracing::info!("Default monitor source: {}", monitor_name);
    Ok(monitor_name)
}

pub fn list_sources() -> Result<Vec<String>> {
    let pactl_out = Command::new("pactl")
        .args(["list", "sources", "short"])
        .output()
        .context("Failed to run pactl list sources")?;

    let stdout = String::from_utf8_lossy(&pactl_out.stdout);
    let sources: Vec<String> = stdout
        .lines()
        .filter(|l| l.contains(".monitor"))
        .filter_map(|l| l.split_whitespace().nth(1).map(String::from))
        .collect();

    Ok(sources)
}
