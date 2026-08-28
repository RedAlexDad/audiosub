# Сборка из исходников

## Linux

### Зависимости

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Системные пакеты
sudo apt install libpulse-dev cmake clang
```

### Сборка

```bash
git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub
cargo build --release
# Бинарник: target/release/audiosub
```

## Windows

### Через MSVC (рекомендуется)

```powershell
# Установить Rust: https://rustup.rs
# Установить Visual Studio Build Tools с C++ workload

git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub
cargo build --release
# Бинарник: target\release\audiosub.exe
```

### Через mingw (кросс-компиляция с Linux)

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# Бинарник: target/x86_64-pc-windows-gnu/release/audiosub.exe
```

## macOS

```bash
# Установить Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub
cargo build --release
# Бинарник: target/release/audiosub
```

## Кросс-платформенное аудио

audiosub использует разные бэкенды в зависимости от ОС:

| ОС | Бэкенд | Захват системного аудио |
|----|--------|------------------------|
| Linux | PulseAudio | ✅ встроенный монитор |
| Windows | CPAL/WASAPI | ❌ требуется VB-Cable |
| macOS | CPAL/CoreAudio | ❌ требуется BlackHole |

На Linux системный звук (YouTube, плеер) захватывается автоматически через PulseAudio monitor.

На Windows и macOS — только микрофон. Для захвата системного аудио установите виртуальный аудиокабель (см. [installation.md](installation.md)).

## Флаги сборки

| Флаг | По умолч. | Описание |
|------|-----------|----------|
| `whisper` | да | Whisper.cpp бэкенд (compile-time) |
| `tui` | да | Терминальный интерфейс (ratatui + crossterm) |

Vosk **не является флагом сборки** — он загружается runtime через `libloading` если есть `libvosk.so`.

### Варианты сборки

```bash
# По умолчанию (whisper + tui)
cargo build

# Минимальная (без TUI, без whisper — только Vosk runtime)
cargo build --no-default-features

# Все возможности (whisper + tui)
cargo build --features whisper,tui
```

## GPU (CUDA)

Whisper можно ускорить на NVIDIA GPU. У крейта своей фичи `cuda` нет —
она включается на зависимости `whisper-rs` в `Cargo.toml`:

```toml
[dependencies.whisper-rs]
version = "0.14"
features = ["default", "cuda", "tracing_backend"]
```

Сборка с CUDA-тулчейном:

```bash
CUDACXX=/usr/local/cuda-13.2/bin/nvcc CUDA_PATH=/usr/local/cuda-13.2 cargo build
```

Ограничения:

- **Драйвер — потолок 13.2**: CUDA 13.3 требует драйвер ≥610.43.
  На драйвере 595.84 максимум — CUDA 13.2 Update 1 (deb-пакеты).
- **CPU fallback**: если в рантайме нет GPU/драйвера/CUDA-либ, whisper
  сам падает на CPU (ggml не регистрирует CUDA-устройств) — код менять
  не нужно, бинарник одинаковый.

## Сборка в Docker

```bash
make docker-build              # ENGINE=vosk (с Vosk SDK)
make docker-build ENGINE=whisper  # Только Whisper, меньший образ
```

## Структура проекта

```
src/
├── main.rs              # Точка входа, CLI → TUI или сессия
├── cli.rs               # Флаги командной строки (clap)
├── config.rs            # Загрузка конфига + динамические defaults
├── session/
│   ├── mod.rs           # Оркестрация сессии + выбор движка
│   └── model.rs         # Поиск модели + авто-детект + загрузка
├── asr/
│   ├── mod.rs           # AsrEngine trait
│   ├── vosk_dl.rs       # Vosk runtime загрузчик (libloading)
│   ├── vosk_backend.rs  # Vosk engine реализация
│   └── whisper_backend.rs
├── audio/
│   ├── mod.rs
│   ├── pulse.rs         # Захват аудио через PulseAudio
│   └── monitor.rs       # Авто-детект устройств
├── subtitle/
│   ├── mod.rs
│   ├── buffer.rs        # Буфер субтитров с объединением
│   ├── split.rs         # Разбивка сегментов
│   ├── srt.rs           # SRT writer
│   └── vtt.rs           # VTT writer
└── tui/
    ├── worker.rs        # 3-поточная оркестрация
    ├── app.rs           # Состояние TuiApp
    ├── event.rs         # Обработка ввода
    ├── screen.rs        # Enum экранов
    └── view/            # Отрисовка (шапка, recognition, сегменты, логи)
```

## Архитектура

```
Поток захвата → mpsc[AudioData] → Поток ASR → mpsc[UiUpdate] → Поток TUI
      ↕ ↕ ↕ Arc<AtomicBool> (stop/pause/reset) ↕ ↕ ↕
```

## Проверка

```bash
make verify   # test → check → clippy → fmt
```
