# Project Instructions for OpenCode

## Quick reference

```sh
make verify          # test -> check -> lint -> fmt (CI pipeline)
make test            # cargo test (SHOW_DESCRIBE=1 for --show-output)
make run             # LD_LIBRARY_PATH=... cargo run --features vosk,tui
make run-whisper     # WHISPER_DONT_GENERATE_BINDINGS=1 cargo run --features whisper,tui --no-default-features
make build-whisper   # WHISPER_DONT_GENERATE_BINDINGS=1 cargo build --no-default-features --features whisper,tui
make build-both      # cargo build --features "vosk,whisper,tui"
```

## Build quirks

- **Vosk runtime:** needs `libvosk.so` in `LD_LIBRARY_PATH` (see `.cargo/config.toml` for rpath)
- **Whisper build:** set `WHISPER_DONT_GENERATE_BINDINGS=1` (bindgen crashes without it)
- **Feature flags:** `default = ["vosk", "tui"]`, CI uses `--no-default-features`
- **System dep:** `libpulse-dev` for PulseAudio capture
- **rustfmt:** max_width=120, tab_spaces=4, edition=2024
- **Lints:** `unsafe_code = "deny"`

## GPU (CUDA)

- **Включение:** фича `cuda` стоит на зависимости `whisper-rs` в Cargo.toml
  (у самого крейта своей фичи нет), сборка — обычный `cargo build`.
- **Toolkit:** CUDA 13.2 Update 1 (deb-пакеты; cublas 13.2.2.2 прилинкован в
  `/usr/local/cuda-13.2/targets/x86_64-linux/`). Драйвер 595.84 = потолок
  13.2: CUDA 13.3 требует драйвер ≥610.43.
- **Сборка:** `CUDACXX=/usr/local/cuda-13.2/bin/nvcc CUDA_PATH=/usr/local/cuda-13.2 cargo build`
- **CPU fallback:** без GPU/драйвера/CUDA-либ в рантайме whisper сам падает
  на CPU (ggml не регистрирует 0 CUDA-устройств) — код менять не нужно.
- **Скачивания:** после рефакторинга HF/`curl` зависают на HTTP/2 — везде
  используется `curl -4 --http1.1` (вшито в `download_model`).

## Testing

- **74 total**: 48 inline (`#[cfg(test)] mod tests`), 26 integration (`tests/*.rs`)
- **Integration suites**: `concurrent_flags` (6), `config_loading` (6), `perf_buffer` (6), `subtitle_pipeline` (8)
- **Perf tests**: `std::time::Instant` asserts with per-iteration thresholds (ns/µs)
- **No dev-dependencies**: criterion not used; all perf via `#[test]` in `tests/perf_buffer.rs`
- Russian `Описание:` via `println!` — show with `cargo test -- --show-output` or `make test SHOW_DESCRIBE=1`
- Pass test binary args after `--` separator

## Architecture

```
Capture thread → mpsc[AudioData] → ASR thread → mpsc[UiUpdate] → TUI thread
      ↕ ↕ ↕ Arc<AtomicBool> (stop/pause/reset) ↕ ↕ ↕
```

- `src/main.rs` — entrypoint, chooses CLI (`run_session()`) or TUI (`tui::worker::run_tui()`)
- `src/tui/worker.rs` — 3-thread orchestration (capture/ASR/TUI)
- `src/tui/capture.rs` — legacy single-threaded path
- Models: предзагруженные в `models/` (gitignored): `vosk-model-small-ru-0.22`, `ggml-{tiny,base,small,medium}.bin`
- Скачивание: `audiosub --download-model whisper:large` → `ggml-large-v3.bin` в текущей директории (`ggml-large.bin` удалён с HF, v1 — отдельный файл)
- Runtime config: `audiosub.toml` at project root (also searchable via CLI arg, CWD, `~/.cache/audiosub/`)
- CLI mode: `make cli` or `cargo run -- --no-tui`
- Output: `subtitles.srt` (configurable), exported copies in `saved/`

## Config

`.env` overrides config: `AUDIOSUB_DEVICE`, `AUDIOSUB_MODEL`, etc. (see `clap` derive with `env` attribute).

## Commit workflow

Перед каждым коммитом:

1. Создать отчёт в `reports/<дата_время>.md`:
   - **What was added** / **What was changed** / **Problems encountered** / **How they were solved**
2. Добавить отчёт в индекс вместе со всеми изменёнными файлами.
3. Выполнить коммит (один раз, без amend).

Формат коммита (только русский):

```
<type>(<опциональная область>): <описание>
```

Типы: feat, fix, docs, style, refactor, test, chore
