# Building from source

## Prerequisites

```bash
# Rust toolchain (see rust-toolchain.toml for pinned version)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies
sudo apt install libpulse-dev cmake clang
```

## Build

```bash
# Clone
git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub

# Build (release, whisper + vosk runtime)
make release
# Binary at: release/audiosub

# Or via cargo directly
cargo build --release
```

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `whisper` | yes | Whisper.cpp backend (compile-time) |
| `tui` | yes | Terminal UI (ratatui + crossterm) |

Vosk is **not a feature** — it's loaded at runtime via `libloading` if `libvosk.so` is present.

### Build variants

```bash
# Default (whisper + tui)
cargo build

# Minimal (no TUI, no whisper — Vosk only via runtime)
cargo build --no-default-features

# With all features (whisper + tui)
cargo build --features whisper,tui
```

## Build with Docker

```bash
make docker-build      # ENGINE=vosk (default, includes Vosk SDK)
make docker-build ENGINE=whisper   # Whisper only, smaller image
```

## Project structure

```
src/
├── main.rs              # Entry point, CLI → TUI or session
├── cli.rs               # CLI flags via clap
├── config.rs            # Config loading + dynamic defaults
├── session/
│   ├── mod.rs           # Session orchestration + engine selection
│   └── model.rs         # Model resolution + auto-detect + download
├── asr/
│   ├── mod.rs           # AsrEngine trait
│   ├── vosk_dl.rs       # Vosk runtime loader (libloading)
│   ├── vosk_backend.rs  # Vosk engine implementation
│   └── whisper_backend.rs
├── audio/
│   ├── mod.rs
│   ├── pulse.rs         # PulseAudio capture
│   └── monitor.rs       # Device auto-detection
├── subtitle/
│   ├── mod.rs
│   ├── buffer.rs        # Subtitle buffer with overlap merge
│   ├── split.rs         # Segment splitting
│   ├── srt.rs           # SRT writer
│   └── vtt.rs           # VTT writer
└── tui/
    ├── worker.rs        # 3-thread orchestration
    ├── app.rs           # TuiApp state
    ├── event.rs         # Input handling
    ├── screen.rs        # Screen enum
    └── view/            # Renderers (top, recognition, segments, logs)
```

## Architecture

```
Capture thread → mpsc[AudioData] → ASR thread → mpsc[UiUpdate] → TUI thread
      ↕ ↕ ↕ Arc<AtomicBool> (stop/pause/reset) ↕ ↕ ↕
```

## Verify

```bash
make verify   # test → check → clippy → fmt
```
