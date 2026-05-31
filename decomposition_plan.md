# План декомпозиции: audiosub

Автоматические субтитры в реальном времени для Ubuntu.
Захват звука ОС → распознавание речи (Vosk / Whisper.cpp) → вывод субтитров.

---

## 1. Инфраструктура проекта

| Задача | Описание |
|--------|----------|
| 1.1 | [x] Инициализация Rust-проекта (Cargo.toml, workspace, lint/format настройки) |
| 1.2 | [x] Настройка CI (GitHub Actions: сборка, clippy, test) |
| 1.3 | [x] Конфигурация проекта (cli-аргументы + config файл + env) |
| 1.4 | [x] Логирование (tracing) |

## 2. Захват аудио системы (Audio Capture)

**Суть:** Перехват системного аудиовыхода (то, что слышат колонки/наушники).

| Задача | Описание |
|--------|----------|
| 2.1 | [x] PulseAudio `monitor` — выбран как primary API |
| 2.2 | [x] Трейт `AudioCapture` с методом `fn read() -> AudioChunk` |
| 2.3 | [x] **Реализация PulseAudio:** libpulse-binding — захват с `monitor` |
| 2.4 | [-] **Реализация PipeWire:** pipewire-rs — отложено |
| 2.5 | [-] **ALSA fallback:** отложено |
| 2.6 | [x] **Перекодировка:** ресемплинг в 16kHz mono f32 (rubato::Fft) |
| 2.7 | [x] **Выбор устройства:** `pactl list sources short` / `--list-devices` |

## 3. Движок распознавания речи (ASR Engine)

**Суть:** Абстрактный слой над Vosk / Whisper.cpp.

| Задача | Описание |
|--------|----------|
| 3.1 | [x] **Абстракция:** трейт `AsrEngine` с методами `feed_audio`, `partial_text`, `drain_segments` |
| 3.2 | [ ] **Загрузка моделей:** авто-загрузка по URL, кэширование в `~/.cache/audiosub/models` |
| 3.3 | [x] **Backend: Vosk API:** `vosk-rs`, инкрементальное распознавание |
| 3.4 | [x] **Backend: Whisper.cpp:** `whisper-rs`, `make build-whisper` / `make build-both` |
| 3.5 | [ ] **Backend: Whisper.cpp (реальное время):** инкрементальный режим (слияние окон) |
| 3.6 | [x] **Переключение движков:** `cargo build --features vosk,whisper` |
| 3.7 | [ ] **Язык:** аргумент `--lang` / автодетект языка |

## 4. Генерация субтитров (Subtitle Pipeline)

**Суть:** Сегменты распознанного текста → форматированные субтитры с таймкодами.

| Задача | Описание |
|--------|----------|
| 4.1 | [x] **Сегментация:** сырые гипотезы → стабильные сегменты с временными метками |
| 4.2 | [x] **Наложение таймкодов:** привязка текста к реальному времени системы |
| 4.3 | [x] **Формат SRT:** вывод в стандартный `.srt` |
| 4.4 | [x] **Формат VTT:** вывод в `.vtt` для веб |
| 4.5 | [x] **Потоковый вывод:** запись в файл в реальном времени (append) |
| 4.6 | [x] **Буферизация:** задержка для улучшения качества (показ с offset) |

## 5. CLI / TUI (User Interface)

**Суть:** Три режима — cli-only, интерактивный TUI, live overlay.

| Задача | Описание |
|--------|----------|
| 5.1 | [x] **CLI (clap):** `--model`, `--output`, `--device`, `--max-duration`, `--no-tui` |
| 5.2 | [x] **TUI (ratatui):** режим по умолчанию, `--no-tui` для CLI |
| 5.3 | [x] **VU meter:** индикатор уровня громкости (цветовая шкала) |
| 5.4 | [x] **Экран распознавания:** live-лента распознанного текста |
| 5.5 | [x] **Экран субтитров:** превью текущих сегментов + скролл |
| 5.6 | [x] **Управление:** пауза (`p`), сброс (`r`), экспорт (`s`/`S`) |
| 5.7 | [x] **Логи:** встроенный просмотр stderr в TUI (`Tab`) |
| 5.8 | [ ] **Режим `--overlay`:** оверлейное окно (GTK/wayland) — stretch goal |

## 6. Обработка ошибок и надежность

| Задача | Описание |
|--------|----------|
| 6.1 | [ ] Graceful degradation при потере аудиопотока |
| 6.2 | [ ] Перезапуск ASR при падении процесса whisper.cpp |
| 6.3 | [ ] Восстановление после сбоев (recovery) |
| 6.4 | [x] Логирование всех этапов (tracing + файл `/tmp/audiosub_stderr.log`) |

## 7. Сборка и дистрибуция

| Задача | Описание |
|--------|----------|
| 7.1 | [x] **Makefile:** `make build/test/run/verify/lint`, `make build-both/build-whisper` |
| 7.2 | [x] **Dockerfile:** мультистейдж-сборка, ENGINE=vosk|whisper|both |
| 7.3 | [x] **Docker Compose:** PulseAudio cookie + модель + /dev/snd |
| 7.4 | [ ] Пакет для Ubuntu (`.deb`) |
| 7.5 | [ ] AppImage / статическая сборка |
| 7.6 | [ ] Документация: README, примеры, скриншоты |

---

## Приоритетная карта (MVP)

**Phase 1 — Ядро**
- [x] 1.1, 1.2, 1.3, 1.4 — каркас проекта
- [x] 2.1, 2.2, 2.3 — захват PulseAudio monitor
- [x] 2.6 — ресемплинг
- [x] 3.1, 3.3 — Vosk backend
- [x] 4.1, 4.2, 4.3, 4.5 — базовый SRT вывод
- [x] 5.1, 5.2 — Basic CLI + TUI

**Phase 2 — Whisper.cpp**
- [x] 3.4 — Whisper.cpp backend
- [x] 3.6 — переключение движков (`cargo build --features vosk,whisper`)

**Phase 3 — Полировка**
- [x] 4.4 — VTT
- [x] 5.3–5.7 — расширенный TUI (VU meter, pause, reset, log viewer)
- [x] 2.7 — выбор устройства
- [x] 4.6 — буферизация
- [x] 6.4 — логирование
- [ ] 6.1–6.3 — надежность

**Phase 3b — Многопоточность**
- [x] 3 потока: Capture → ASR → TUI (mpsc channels)
- [x] `Arc<AtomicBool>` — stop/pause/reset
- [x] ASR thread — resample + engine + buffer + output

**Phase 4 — Дистрибуция**
- [x] 7.1 — Makefile
- [x] 7.2, 7.3 — Docker + Compose
- [ ] 7.4–7.6 — упаковка, документация

---

## Стек технологий

| Компонент | Библиотека |
|-----------|------------|
| CLI аргументы | `clap` (derive + env) |
| TUI | `ratatui` + `crossterm` (feature `tui`) |
| Аудио захват | `libpulse-binding` + `libpulse-simple-binding` |
| Ресемплинг | `rubato` (FftFixed + FixedSync) |
| ASR Vosk | `vosk-rs` (feature `vosk`) |
| ASR Whisper | `whisper-rs` (feature `whisper`) |
| Асинхронность | `tokio` (full) |
| Конфиг | `serde` + `toml` + `directories` |
| Таймкоды | `chrono` |
| Логи | `tracing` + `tracing-subscriber` + `tracing-appender` |
| Ошибки | `anyhow` + `thiserror` |
| Загрузка моделей | `reqwest` (опционально) |
| Утилиты | `duct` (pactl), `libc` |
| Сборка | `make` (Makefile) |
| CI/CD | `GitHub Actions` (`.github/workflows/`) |
