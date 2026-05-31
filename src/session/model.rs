use std::path::{Path, PathBuf};

pub fn resolve_model_path(cli_path: Option<&PathBuf>, cfg_path: &Path) -> String {
    let base = cli_path
        .cloned()
        .or_else(|| Some(cfg_path.to_path_buf()))
        .unwrap_or_else(default_model_path);
    let abs = if base.is_absolute() {
        base
    } else {
        std::env::current_dir().unwrap_or_default().join(&base)
    };
    abs.to_string_lossy().to_string()
}

fn default_model_path() -> PathBuf {
    let cache_dir = directories::BaseDirs::new()
        .map(|d| d.cache_dir().join("audiosub").join("models"))
        .unwrap_or_else(|| PathBuf::from("~/.cache/audiosub/models"));
    cache_dir.join("vosk-model-small-en-us-0.15")
}
