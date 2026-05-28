use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
    pub asr: AsrConfig,
    pub subtitle: SubtitleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    pub engine: String,
    pub model_path: PathBuf,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    pub format: String,
    pub output: PathBuf,
    pub buffer_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                device: "default".into(),
                sample_rate: 16000,
                channels: 1,
            },
            asr: AsrConfig {
                engine: "vosk".into(),
                model_path: dirs().join("models"),
                lang: "en-US".into(),
            },
            subtitle: SubtitleConfig {
                format: "srt".into(),
                output: PathBuf::from("output.srt"),
                buffer_ms: 2000,
            },
        }
    }
}

fn dirs() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.cache_dir().join("audiosub"))
        .unwrap_or_else(|| PathBuf::from("/tmp/audiosub"))
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        if let Some(p) = path {
            let content = std::fs::read_to_string(p).context(format!("Failed to read config from {:?}", p))?;
            Ok(toml::from_str(&content)?)
        } else {
            let default_path = dirs().join("config.toml");
            if default_path.exists() {
                let content = std::fs::read_to_string(&default_path)?;
                Ok(toml::from_str(&content)?)
            } else {
                Ok(Self::default())
            }
        }
    }
}
