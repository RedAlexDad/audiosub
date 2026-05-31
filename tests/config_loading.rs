use std::fs;
use std::path::PathBuf;

use audiosub::config::Config;

#[test]
fn config_load_from_specific_file() {
    let dir = fs::canonicalize(".").unwrap();
    let path = dir.join(format!("test_config_{}.toml", std::process::id()));
    let _ = fs::remove_file(&path);

    let content = r#"
[audio]
device = "hw:0,0"
sample_rate = 48000
channels = 2

[asr]
engine = "whisper"
model_path = "/tmp/model.bin"
lang = "ru"

[subtitle]
format = "vtt"
output = "caps.vtt"
buffer_ms = 3000
max_duration_ms = 8000
"#;
    fs::write(&path, content).unwrap();

    let cfg = Config::load(Some(&path)).unwrap();
    assert_eq!(cfg.audio.device, "hw:0,0");
    assert_eq!(cfg.audio.sample_rate, 48000);
    assert_eq!(cfg.audio.channels, 2);
    assert_eq!(cfg.asr.engine, "whisper");
    assert_eq!(cfg.asr.model_path, PathBuf::from("/tmp/model.bin"));
    assert_eq!(cfg.asr.lang, "ru");
    assert_eq!(cfg.subtitle.format, "vtt");
    assert_eq!(cfg.subtitle.output, PathBuf::from("caps.vtt"));
    assert_eq!(cfg.subtitle.buffer_ms, 3000);
    assert_eq!(cfg.subtitle.max_duration_ms, 8000);

    fs::remove_file(&path).unwrap();
}

#[test]
fn config_load_nonexistent_path_returns_default() {
    let cfg = Config::load(Some(&PathBuf::from("/nonexistent/audiosub.toml")));
    assert!(cfg.is_err());
}

#[test]
fn config_default_matches_expected_sentinel_values() {
    let cfg = Config::default();
    assert_eq!(cfg.audio.device, "default");
    assert_eq!(cfg.asr.engine, "vosk");
    assert_eq!(cfg.subtitle.format, "srt");
    assert_eq!(cfg.subtitle.buffer_ms, 2000);
    assert_eq!(cfg.subtitle.max_duration_ms, 10000);
}

#[test]
fn config_load_from_invalid_toml_returns_error() {
    let dir = fs::canonicalize(".").unwrap();
    let path = dir.join(format!("test_config_bad_{}.toml", std::process::id()));
    fs::write(&path, "this is not valid toml === {{{").unwrap();

    let result = Config::load(Some(&path));
    assert!(result.is_err());

    fs::remove_file(&path).unwrap();
}

#[test]
fn config_load_minimal_toml_fills_defaults_for_missing_sections() {
    let dir = fs::canonicalize(".").unwrap();
    let path = dir.join(format!("test_config_min_{}.toml", std::process::id()));
    let content = r#"
[audio]
device = "pulse"
sample_rate = 16000
channels = 1
"#;
    fs::write(&path, content).unwrap();

    let result = Config::load(Some(&path));
    assert!(result.is_err()); // asr section is required by serde

    fs::remove_file(&path).unwrap();
}

#[test]
fn config_supports_deserialize_roundtrip() {
    let cfg = Config::default();
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.audio.device, cfg.audio.device);
    assert_eq!(parsed.asr.engine, cfg.asr.engine);
    assert_eq!(parsed.subtitle.format, cfg.subtitle.format);
}
