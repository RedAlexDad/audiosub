# Оценка уровня программиста по кодовой базе audiosub

**Дата:** 2026-06-01
**Контекст:** Код написан через AI (vibe-coding), но архитектура, ревью и интеграция сделаны человеком.

---

## Итоговая оценка: Senior (уверенный сеньор), 6.5-7/10

---

## 1. Архитектура (сеньор)

- **3-поточная схема** с mpsc-каналами и `Arc<AtomicBool>` для синхронизации — нетривиально, требует понимания Send/Sync, гонок и graceful shutdown
- **Trait-based дизайн:** `AudioCapture`, `AsrEngine`, `SubtitleWriter` — абстракция над Vosk/Whisper/SRT/VTT, добавление новых бекендов без изменения кода
- **SubtitleBuffer** с `flush()` cutoff + merge overlapping segments — нетривиальный алгоритм с корректным `saturating_sub`
- **Factory-паттерн** для создания ASR-движка (`create_engine`)
- Feature-gated модули (`#[cfg(feature = "tui")]`, `#[cfg(feature = "vosk")]`)

## 2. Rust-идиомы (сеньор)

- `std::mem::take()`, `saturating_sub()`, `div_ceil()`, `unwrap_or_default()`
- `let-else` с паттерн-матчингом
- Изолированный `rustfmt.toml` (max_width=120, tab_spaces=4, edition=2024)
- `[lints.rust] unsafe_code = "deny"`
- `.cargo/config.toml` с rpath для локальных библиотек
- Builder-паттерн (`with_target_rate`)

## 3. Domain expertise (сеньор)

- PulseAudio capture (FLOAT32NE, monitor source, Simple API) + resampling через rubato
- Vosk API с word-level timestamps и обработкой `DecodingState::Finalized`
- Whisper.cpp FFI с буферизацией и sliding window (3s update interval)
- SRT/VTT format: `ms_to_srt()` корректно обрабатывает 25+ часов
- Cross-compilation: Makefile цели для Windows (mingw-w64) и macOS

## 4. DevOps & Tooling (сеньор)

- GitHub Actions CI c `--no-default-features`
- Docker multi-stage build с ARG ENGINE (vosk/whisper/both)
- `.env` + clap derive с env-атрибутами
- Makefile с 30+ целями, цветной вывод, моделирование CI
- Настройки для VSCode и Zed

## 5. Тестирование (middle → senior)

- 48 unit-тестов в 6 модулях
- Хорошее покрытие edge cases: empty, zero, boundary, overflow, pause, duplicate, clip
- `assert!` с tolerance для float (`1e-6`)
- Описания тестов на русском через `println!("Описание: ...")`

**Чего не хватало на момент написания:**
- Только inline unit-тесты, нет `tests/` директории (integration tests)
- Нет тестов для многопоточности (например, тестов гонок в worker)
- Нет бенчмарков для latency-чувствительного кода

## 6. Что мешает поставить 8-9/10 (минусы)

| Проблема | Серьёзность |
|----------|-------------|
| `#![allow(dead_code)]` на весь крейт | Средняя |
| `unsafe impl Send for PulseCapture` (хоть с комментарием) | Средняя |
| Дублирование `compute_rms`/`compute_peak` в worker.rs и app.rs | Низкая |
| Нет integration tests | Средняя |
| `engine.finalize()` → `unwrap_or_default()` (тихая потеря ошибок) | Низкая |
| Legacy capture.rs дублирует worker.rs | Низкая |

## 7. Вклад человека vs AI

| Что сделал AI | Что сделал человек |
|---------------|-------------------|
| Синтаксис, имплементации, тесты | Архитектура модулей и трейтов |
| Код-генерация функций | Выбор стека (PulseAudio, Vosk, ratatui, rubato) |
| Написание 48 тестов | CI/CD: Makefile, Docker, GitHub Actions |
| | Ревью: фильтрация AI-ошибок и чушь-генерации |
| | Интеграция: сборка разнородных компонентов |
| | Тредовая модель: mpsc + Arc<AtomicBool> |

**Вывод:** AI генерирует строки, но архитектуру, границы ответственности, стратегию обработки ошибок и потоковую модель собирает человек. Навык декомпозиции, выбора инструментов и ревью — это Senior-скилл, который не появляется от промптов.

---

## Шкала

| Уровень | Грейд | Описание |
|---------|-------|----------|
| Junior | 1-3 | Пишет код, но не видит архитектуры |
| Middle | 4-5 | Пишет хороший код, но не строит систему |
| Senior | 6-7 | Строит систему, выбирает стек, ревьюит |
| Lead | 8-9 | Ведёт направление, учит других |
| Architect | 10 | Видит всю картинку кросс-проектно |
