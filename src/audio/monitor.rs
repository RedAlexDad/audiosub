use anyhow::{Context, Result};

/// Ищет устройство для захвата системного аудио.
///
/// Приоритет:
/// 1. Linux → PulseAudio monitor дефолтного вывода
/// 2. Windows → VB-Cable / CABLE Output
/// 3. macOS → BlackHole
/// 4. Любое другое — дефолтный input
pub fn find_default_monitor() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
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

    #[cfg(not(target_os = "linux"))]
    {
        use cpal::traits::{DeviceTrait, HostTrait};
        let keywords = &["CABLE Output", "VB-Cable", "BlackHole", "VoiceMeeter"];
        let host = cpal::default_host();
        if let Ok(devices) = host.input_devices() {
            for device in devices.flatten() {
                if let Ok(desc) = device.description() {
                    let name = desc.name();
                    if keywords.iter().any(|k| name.contains(k)) {
                        tracing::info!("Found loopback device: {name}");
                        return Ok(name.to_string());
                    }
                }
            }
        }
        anyhow::bail!("No loopback device found. Install VB-Cable (Windows) or BlackHole (macOS)")
    }
}

pub fn list_sources() -> Result<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
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

    #[cfg(not(target_os = "linux"))]
    {
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        let sources: Vec<String> = host
            .input_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(sources)
    }
}
