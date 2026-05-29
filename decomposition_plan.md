# decomposition_plan.md

## 1. Done

### Core Audio Capture
- [x] PulseAudio monitor capture (`PulseCapture`) — system audio via ALSA monitor
- [x] Audio resampler (`AudioResampler`, `rubato::Fft` + `FixedSync::Both`)
- [x] `AudioCapture` trait for abstraction
- [x] Monitor device detection
- [x] CLI `--list-devices` flag

### ASR Backend (Vosk)
- [x] Vosk backend (`AsrEngine` trait, feature `#[cfg(feature = "vosk")]`)
- [x] Incremental recognition: `feed_audio`, `partial_text`, `drain_segments`
- [x] Word-level timestamps from Vosk

### Subtitle Output
- [x] `SubtitleWriter` — streaming SRT/VTT via `BufWriter`
- [x] `SubtitleBuffer` — delay by `buffer_ms`, merge overlapping segments
- [x] `split_segment()` — split long segments by `max_duration_ms`
- [x] `SubtitleOutput` — shared writer with `Arc<Mutex<>>`

### TUI (ratatui + crossterm)
- [x] Full-history display (no cap)
- [x] Scrolling: `↑`/`↓`, `PgUp`/`PgDown`, `Home`/`End`
- [x] Pause mode (`p`) — freeze display for text copy
- [x] Export: `s` → SRT, `S` (Shift+S) → TXT to `saved/`
- [x] Save confirmation message (15 frames, auto-clear)
- [x] Quit: `q`/`Esc`/`Ctrl+D` (Ctrl+C removed for terminal copy)
- [x] TUI is default mode; `--no-tui` for CLI

### Configuration & CLI
- [x] `audiosub.toml` — typed TOML sections `[audio]`, `[asr]`, `[subtitle]`
- [x] `Config::load()` — CWD → `~/.config/audiosub/` → defaults
- [x] CLI overrides TOML overrides defaults
- [x] `resolve_model_path()` — relative → absolute (Vosk requirement)
- [x] Only `AUDIOSUB_CONFIG` env var remains (config file path)
- [x] `--max-duration`, `--no-tui`, `--list-devices`, `--duration` flags

### Build & CI
- [x] GitHub Actions (`make ci-check`)
- [x] Native build workflow (`make build`, `make run`, `make model-download`)
- [x] Dockerfile (legacy builder, no BuildKit)
- [x] `make verify` — test + check + lint + fmt
- [x] All clippy warnings fixed

### Model
- [x] Model download via `make model-download`
- [x] `models/vosk-model-small-ru-0.22` downloaded and working

### Reports & Docs
- [x] Reports in `reports/YYYY-MM-DD/HH-MM-SS.md`
- [x] AGENTS.md with commit workflow

## 2. In Progress

- (none)

## 3. Next

### Whisper.cpp Backend
- [ ] `#[cfg(feature = "whisper")]` module
- [ ] Integration with whisper.cpp bindings

### Extended TUI Features
- [ ] VU meter / audio level indicator
- [ ] Pause/reset button
- [ ] Log viewer for debug output

### Documentation
- [ ] README with setup and usage
- [ ] ARCHITECTURE.md

### Polish
- [ ] Error handling (user-friendly messages)
- [ ] Graceful handling of missing model/library
