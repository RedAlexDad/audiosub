# Report 2026-06-01

## What was changed

- `#![allow(dead_code)]` removed from `src/main.rs`
- `AudioChunk` struct stripped of unused `timestamp` and `sample_rate` fields in `src/audio/mod.rs`
- `PulseCapture::with_target_rate()` removed in `src/audio/pulse.rs`
- `AudioResampler` stripped of unused `input_rate`, `output_rate` fields and their getters (including `input_needed()`) in `src/audio/resampler.rs`
- `SubtitleOutput::append_all()` removed in `src/subtitle/output.rs`
- `VttWriter::new()` removed (Default impl suffices) in `src/subtitle/vtt.rs`
- Legacy single-thread `src/tui/capture.rs` deleted entirely
- `pub mod capture` removed from `src/tui/mod.rs`
- `TuiApp` stripped of `is_running()`, `reset_requested()`, `update_audio()`, `update_audio_levels()` in `src/tui/app.rs`
- Duplicate `compute_rms`/`compute_peak` functions and their tests removed from `src/tui/app.rs`
- `compute_rms`/`compute_peak` promoted to `pub(crate)` and their tests moved to `src/tui/worker.rs`
- Integration test `tests/subtitle_pipeline.rs` updated to use loops instead of removed `append_all()`
- `#[allow(dead_code)]` added to a false-positive test function in `src/config.rs`

## Problems encountered

- The test function `default_config_has_expected_values` in `config.rs` triggers a dead_code false-positive in the test binary despite being annotated with `#[test]` inside a `#[cfg(test)]` module
- A `remove_dir_all` error occurred during `cargo clean` (pre-existing, not related to these changes)

## How they were solved

- The false-positive was suppressed with `#[allow(dead_code)]` on the specific test function
- All other dead code was deleted outright — no `#[allow]` annotations were used, ensuring zero dead code in the production build
