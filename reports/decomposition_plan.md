# План декомпозиции: "audiosub работает из коробки"

## Мотивация
Сейчас бинарник требует `audiosub.toml` с путями к модели и устройству. Цель — чтобы после `unzip audiosub.zip && ./audiosub` всё работало без конфигов, флагов и доп. действий.

---

## Задача 1: Модель — автопоиск + авто-загрузка

### 1.1 Автопоиск модели (src/session/model.rs)

В `resolve_model_path()` добавить **последний fallback** после CLI/конфига:

```rust
fn auto_detect_model() -> Option<PathBuf> {
    // 1. Директория бинарника
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Some(found) = scan_for_model(dir) {
                return Some(found);
            }
        }
    }
    // 2. Текущая директория
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(found) = scan_for_model(&cwd) {
            return Some(found);
        }
    }
    None
}

fn scan_for_model(dir: &Path) -> Option<PathBuf> {
    // Ищем файлы .gguf, .bin, .ggml
    for entry in std::fs::read_dir(dir).ok()? {
        let path = entry.ok()?.path();
        if let Some(ext) = path.extension() {
            match ext.to_str()? {
                "gguf" | "bin" | "ggml" => return Some(path),
                _ => continue,
            }
        }
    }
    None
}
```

**Проверка загруженности модели:** после `load_model()` проверять, что модель действительно загрузилась. Если нет — падать с понятной ошибкой и подсказкой `--download-model`.

### 1.2 Авто-загрузка (CLI флаг + код)

**Новый флаг CLI** (src/cli.rs):
```rust
#[arg(long, help = "Download ASR model: vosk | whisper (default: auto)")]
pub download_model: Option<Option<String>>,
```

- `--download-model` без аргумента → авто-выбор
- `--download-model vosk` → принудительно Vosk
- `--download-model whisper` → принудительно Whisper
- `--download-model` несовместим с запуском захвата (сразу выходит после загрузки)

**Логика выбора модели для загрузки:**
- Если libvosk.so доступен → Vosk small ru (потому что он ~42MB, распознаёт русский)
- Если libvosk.so нет → Whisper tiny (потому что он ~75MB, а base ~147MB)
- Пользователь может указать конкретный движок флагом

**Код загрузки** (можно в отдельный модуль `src/model_download.rs` или в `src/session/model.rs`):
```rust
pub fn download_model(engine: &str, dir: &Path) -> Result<()> {
    match engine {
        "vosk" => {
            // curl + unzip vosk-model-small-ru-0.22
        }
        "whisper" => {
            // curl ggml-tiny.bin (или ggml-tiny.en.bin)
        }
        _ => bail!("Unknown engine"),
    }
}
```

**Файлы моделей:**
- Vosk: `https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip` (распаковать)
- Whisper tiny: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin`
- Whisper tiny.en: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin`

### 1.3 Отображение имени модели в TUI (src/tui/view/top.rs)

В хедере после имени движка добавить сокращённое имя файла модели:
```
audiosub │ VOSK (small-ru) │ ● RUNNING │ ...
```
или
```
audiosub │ WHISPER (tiny) │ ● RUNNING │ ...
```

Поле `model_name` добавить в `TuiApp`.

---

## Задача 2: Устройство — автодетект с fallback

### 2.1 Текущее состояние

Сейчас (src/main.rs):
```rust
let device = args.device.clone().or_else(|| {
    if cfg.audio.device == "default" {
        audiosub::audio::find_default_monitor().ok()
    } else {
        Some(cfg.audio.device.clone())
    }
}).unwrap_or_else(|| "default".into());
```

### 2.2 Новый fallback (src/main.rs)

```rust
let device = args.device.clone().or_else(|| {
    // Пробуем config
    if cfg.audio.device != "default" {
        return Some(cfg.audio.device.clone());
    }
    // Пробуем default monitor
    if let Ok(mon) = audiosub::audio::find_default_monitor() {
        return Some(mon);
    }
    // Пробуем первый из списка
    if let Ok(sources) = audiosub::audio::list_sources() {
        if let Some(first) = sources.into_iter().next() {
            return Some(first);
        }
    }
    // Последний шанс — "default"
    Some("default".into())
});
```

### 2.3 Если устройство не найдено — подсказка

При ошибке открытия устройства:
```rust
Err(e) => {
    tracing::error!("Failed to open audio device '{device}': {e}");
    tracing::info!("List available devices: audiosub --list-devices");
    return Err(e);
}
```

---

## Задача 3: Динамические defaults в Config

### 3.1 Проблема

`Config::default()` возвращает статичные значения:
```rust
device: "default".into(),
engine: "vosk".into(),
model_path: dirs().join("models"),  // ~/.cache/audiosub/models
```

Но `engine = "vosk"` не имеет смысла без libvosk.so. И `device = "default"` часто не работает как монитор.

### 3.2 Решение (src/config.rs)

Сделать `Config::default()` динамическим:

```rust
impl Default for Config {
    fn default() -> Self {
        let device = find_default_monitor()
            .or_else(|_| list_sources().ok()?.into_iter().next())
            .unwrap_or_else(|| "default".into());

        let engine = if vosk_dl::is_available() {
            "vosk"
        } else {
            "whisper"
        };

        // model_path не заполняем — позже resolve_model_path + auto_detect_model
        Self {
            audio: AudioConfig { device, sample_rate: 16000, channels: 1 },
            asr: AsrConfig {
                engine: engine.into(),
                model_path: PathBuf::from(""),  // Пустой — будет auto-detect
                model_path_vosk: None,
                model_path_whisper: None,
                lang: "ru-RU".into(),
            },
            subtitle: SubtitleConfig { /* ... */ },
        }
    }
}
```

### 3.3 Нюанс

`Config::load()` сейчас возвращает `Config::default()` когда нет конфига. С динамическим default-ом это будет означать:
- Без конфига → авто-детект устройства и движка
- С конфигом → явные настройки из файла

---

## Задача 4: Обновление CI и Makefile

### 4.1 mk/model.mk

Добавить:
```makefile
model-download-tiny:
	@echo "$(CYAN)→ Downloading Whisper tiny...$(NC)"
	mkdir -p $(MODEL_DIR)
	curl -L -o $(MODEL_DIR)/ggml-tiny.bin \
		https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
```

### 4.2 Release workflow

- Убрать Vosk SDK установку (он не нужен при сборке)
- Обновить Dockerfile — бинарник самодостаточный
- В release notes добавить "Quick start"

### 4.3 Dockerfile

Опционально: встроить модель tiny в образ по умолчанию, чтобы `docker run --rm ghcr.io/...` сразу работал.

---

## Приоритет и зависимости

```
Задача 1.1 (автопоиск модели)
    │
    ├──→ Задача 3 (динамический Config)
    │         │
    │         └──→ Задача 2 (автодетект устройства)
    │
    └──→ Задача 1.2 (авто-загрузка)
              │
              └──→ Задача 1.3 (имя модели в TUI)

Задача 4 (CI/Makefile) — независимо, можно в любой момент
```

**Порядок реализации:**
1. Автопоиск модели (1.1) — даёт zero-config для model
2. Автодетект устройства (2) — даёт zero-config для device
3. Динамический Config (3) — объединяет всё в единый default
4. Авто-загрузка модели (1.2) — для случая когда модели нет совсем
5. TUI — имя модели (1.3) — косметика
6. Обновление CI/Docker (4) — поддержка инфраструктуры

---

## Файлы для изменений

| Файл | Что меняется |
|------|-------------|
| `src/session/model.rs` | + `auto_detect_model()`, + `scan_for_model()`, + `download_model()` |
| `src/cli.rs` | + `--download-model` флаг |
| `src/main.rs` | обработка `--download-model`, новый fallback device |
| `src/config.rs` | `Config::default()` с динамическим device/engine |
| `src/audio/monitor.rs` | (возможно) улучшить `find_default_monitor` |
| `src/tui/app.rs` | + `model_name` поле |
| `src/tui/view/top.rs` | отображение `model_name` |
| `src/session/mod.rs` | передача имени модели в TuiApp |
| `mk/model.mk` | + `model-download-tiny` |
| `Dockerfile` | опционально: встроить tiny модель |
