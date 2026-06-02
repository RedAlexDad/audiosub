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
    pub model_path_vosk: Option<PathBuf>,
    pub model_path_whisper: Option<PathBuf>,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    pub format: String,
    pub output: PathBuf,
    pub buffer_ms: u64,
    pub max_duration_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        let device = crate::audio::find_default_monitor()
            .or_else(|_| crate::audio::list_sources().map(|s| s.into_iter().next().unwrap_or_else(|| "default".into())))
            .unwrap_or_else(|_| "default".to_string());

        let engine = crate::session::model::detect_engine();

        Self {
            audio: AudioConfig {
                device,
                sample_rate: 16000,
                channels: 1,
            },
            asr: AsrConfig {
                engine: engine.into(),
                model_path: PathBuf::new(),
                model_path_vosk: None,
                model_path_whisper: None,
                lang: "ru-RU".into(),
            },
            subtitle: SubtitleConfig {
                format: "srt".into(),
                output: PathBuf::from("subtitles.srt"),
                buffer_ms: 2000,
                max_duration_ms: 10000,
            },
        }
    }
}

fn dirs() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.cache_dir().join("audiosub"))
        .unwrap_or_else(|| PathBuf::from("/tmp/audiosub"))
}

const CONFIG_FILE_NAME: &str = "audiosub.toml";

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        if let Some(p) = path {
            let content = std::fs::read_to_string(p).context(format!("Failed to read config from {:?}", p))?;
            return Ok(toml::from_str(&content)?);
        }

        for candidate in &[PathBuf::from(CONFIG_FILE_NAME), dirs().join(CONFIG_FILE_NAME)] {
            if candidate.exists() {
                let content = std::fs::read_to_string(candidate)?;
                return Ok(toml::from_str(&content)?);
            }

            #[cfg(test)]
            mod tests {
                use super::*;

                #[allow(dead_code)]
                #[test]
                fn default_config_has_expected_structure() {
                    println!("Описание: Config::default() возвращает корректную структуру");
                    let cfg = Config::default();
                    assert_eq!(cfg.audio.sample_rate, 16000);
                    assert_eq!(cfg.audio.channels, 1);
                    assert!(matches!(cfg.asr.engine.as_str(), "vosk" | "whisper"));
                    assert_eq!(cfg.subtitle.format, "srt");
                    assert_eq!(cfg.subtitle.buffer_ms, 2000);
                    assert_eq!(cfg.subtitle.max_duration_ms, 10000);
                }
            }
        }

        Ok(Self::default())
    }
}
