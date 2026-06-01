use std::path::PathBuf;

pub fn resolve_model_path(engine: &str, cli_path: Option<&PathBuf>, cfg: &crate::config::AsrConfig) -> String {
    let base = cli_path
        .cloned()
        .or_else(|| engine_path(engine, cfg).or_else(|| Some(cfg.model_path.clone())))
        .unwrap_or_else(default_model_path);
    let abs = if base.is_absolute() {
        base
    } else {
        std::env::current_dir().unwrap_or_default().join(&base)
    };
    abs.to_string_lossy().to_string()
}

fn engine_path(engine: &str, cfg: &crate::config::AsrConfig) -> Option<PathBuf> {
    match engine {
        "vosk" => cfg.model_path_vosk.clone(),
        "whisper" => cfg.model_path_whisper.clone(),
        _ => None,
    }
}

fn default_model_path() -> PathBuf {
    let cache_dir = directories::BaseDirs::new()
        .map(|d| d.cache_dir().join("audiosub").join("models"))
        .unwrap_or_else(|| PathBuf::from("~/.cache/audiosub/models"));
    cache_dir.join("vosk-model-small-en-us-0.15")
}
