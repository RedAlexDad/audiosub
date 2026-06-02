use std::path::{Path, PathBuf};

const VOSK_MODEL_URL: &str = "https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip";
const VOSK_MODEL_DIR: &str = "vosk-model-small-ru-0.22";
const WHISPER_TINY_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin";
const WHISPER_TINY_NAME: &str = "ggml-tiny.bin";

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

pub fn download_model(engine: &str, dir: &Path) -> anyhow::Result<()> {
    match engine {
        "vosk" => download_vosk(dir),
        "whisper" => download_whisper(dir),
        "auto" => {
            if crate::asr::vosk_dl::is_available() {
                download_vosk(dir)
            } else {
                download_whisper(dir)
            }
        }
        _ => anyhow::bail!("Unknown engine '{engine}'. Use 'vosk' or 'whisper'."),
    }
}

fn download_vosk(dir: &Path) -> anyhow::Result<()> {
    let zip_path = dir.join("vosk-model.zip");
    println!("Downloading Vosk model (Russian, ~42 MB)...");
    duct::cmd!("curl", "-sL", "-o", &zip_path, VOSK_MODEL_URL).run()?;
    println!("Extracting...");
    duct::cmd!("unzip", "-qo", &zip_path, "-d", dir).run()?;
    std::fs::remove_file(&zip_path)?;
    println!("✓ Vosk model downloaded to {}", dir.join(VOSK_MODEL_DIR).display());
    println!(
        "  Set engine=\"vosk\" and model_path=\"{}\" in audiosub.toml",
        VOSK_MODEL_DIR
    );
    Ok(())
}

fn download_whisper(dir: &Path) -> anyhow::Result<()> {
    let path = dir.join(WHISPER_TINY_NAME);
    println!("Downloading Whisper tiny model (~75 MB)...");
    duct::cmd!("curl", "-sL", "-o", &path, WHISPER_TINY_URL).run()?;
    println!("✓ Whisper model downloaded to {}", path.display());
    Ok(())
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
