use std::path::{Path, PathBuf};

pub fn resolve_model_path(engine: &str, cli_path: Option<&PathBuf>, cfg: &crate::config::AsrConfig) -> String {
    let base = cli_path
        .cloned()
        .or_else(|| engine_path(engine, cfg).or_else(|| Some(cfg.model_path.clone())))
        .or_else(auto_detect_model)
        .unwrap_or_else(|| default_model_path(engine));
    if base.as_os_str().is_empty() {
        return String::new();
    }
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

fn auto_detect_model() -> Option<PathBuf> {
    let from_exe = std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|p| p.parent())
        .and_then(scan_for_model);
    from_exe.or_else(|| std::env::current_dir().ok().and_then(|cwd| scan_for_model(&cwd)))
}

fn scan_for_model(dir: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && matches!(path.extension().and_then(|e| e.to_str()), Some("gguf" | "bin" | "ggml")) {
                candidates.push(path);
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    candidates.sort();
    Some(candidates[0].clone())
}

fn default_model_path(engine: &str) -> PathBuf {
    if engine == "vosk" && crate::asr::vosk_dl::is_available() {
        let cache_dir = directories::BaseDirs::new()
            .map(|d| d.cache_dir().join("audiosub").join("models"))
            .unwrap_or_else(|| PathBuf::from("~/.cache/audiosub/models"));
        return cache_dir.join("vosk-model-small-ru-0.22");
    }
    PathBuf::from("")
}
