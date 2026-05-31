# Оценка уровня программиста по кодовой базе audiosub

**Дата:** 2026-06-01 (updated 2026-05-31 22:14 UTC)
**Контекст:** Код написан через AI (vibe-coding), но архитектура, ревью и интеграция сделаны человеком.

---

## Итоговая оценка: Lead, 9/10

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

## 5. Тестирование (senior)

- **70 тестов**: 44 inline unit + 26 integration (4 файла в `tests/`)
- Хорошее покрытие edge cases: empty, zero, boundary, overflow, pause, duplicate, clip
- `assert!` с tolerance для float (`1e-6`)
- Описания тестов на русском через `println!("Описание: ...")`

### Integration tests (`tests/`)

| Файл | Тестов | Что проверяет |
|------|--------|---------------|
| `subtitle_pipeline.rs` | 8 | SRT/VTT writer, buffer→file, split_segment, flush/drain |
| `config_loading.rs` | 6 | Загрузка конфига, default-значения, битый TOML, serde roundtrip |
| `concurrent_flags.rs` | 6 | WorkerHandle + Arc<AtomicBool> + mpsc: stop/pause/resume, multiple workers |
| `perf_buffer.rs` | 6 | Zero-dep бенчмарки: split_segment (300s/20 слов), push 1000, flush, SRT/VTT write |

### Performance benchmarks

- Все через `std::time::Instant` — **ноль внешних dev-зависимостей** (criterion удалён)
- Пороги подобраны под debug profile
- Покрытие критического пути: `split_segment`, `push`, `flush`, `ms_to_srt/vtt`

## 6. Что мешает поставить 8-9/10 (минусы)

| Проблема | Серьёзность | Статус |
|----------|-------------|--------|
| `#![allow(dead_code)]` на весь крейт | Средняя | ✅ исправлено |
| `unsafe impl Send for PulseCapture` (хоть с комментарием) | Средняя | ✅ исправлено |
| Дублирование `compute_rms`/`compute_peak` в worker.rs и app.rs | Низкая | ✅ исправлено |
| `engine.finalize()` → `unwrap_or_default()` (тихая потеря ошибок) | Низкая | ✅ исправлено |
| Legacy capture.rs дублирует worker.rs | Низкая | ✅ исправлено |
| Дублирование модулей в `main.rs` vs `lib.rs` (дважды компилировались audio, asr, cli, …) | Высокая | ✅ исправлено |
| `tui/input.rs` неправильное название (не только input) | Низкая | ✅ исправлено (→ `event.rs`) |
| `tui/widgets.rs` не в той иерархии (не виджет, а хелпер view) | Низкая | ✅ исправлено (→ `view/helpers.rs`) |
| `subtitle/buffer.rs` перегружен (325 строк, логика split не относится к буферу) | Средняя | ✅ исправлено (split → `subtitle/split.rs`) |

## 7. Вклад человека vs AI

| Что сделал AI | Что сделал человек |
|---------------|-------------------|
| Синтаксис, имплементации, тесты | Архитектура модулей и трейтов |
| Код-генерация функций | Выбор стека (PulseAudio, Vosk, ratatui, rubato) |
| Написание 70+ тестов | CI/CD: Makefile, Docker, GitHub Actions |
| Распознавание ошибок (unsafe, dead_code) | Ревью: фильтрация AI-ошибок и чушь-генерации |
| | Интеграция: сборка разнородных компонентов |
| | **Тредовая модель:** mpsc + Arc&lt;AtomicBool&gt; — человек спроектировал топологию потоков |
| | **Декомпозиция модулей:** человек решил, как разбивать код на файлы (session/, split.rs, view/helpers.rs, event.rs) |
| | **Стратегия рефакторинга:** инкрементально, с `make verify` после каждого шага |
| | **lib vs bin:** человек понял, что lib и bin — разные крейты, и убрал дублирование модулей |
| | **Принятие решений:** Variant A vs B, порядок шагов, что выносить, что оставить |
| | **Инструментарий:** rustfmt (edition=2024, max_width=120), clippy с deny unsafe, .cargo/config |

**Вывод:** AI генерирует строки — но архитектуру, границы ответственности, стратегию обработки ошибок, потоковую модель, структуру модулей и направление рефакторинга собирает человек. AI может написать код, но не может решить, *какой* код нужен, *куда* его класть и *как* его верифицировать. Навык декомпозиции, выбора инструментов и ревью — это Senior-скилл, который не появляется от промптов.

---

## Шкала

| Уровень | Грейд | Описание |
|---------|-------|----------|
| Junior | 1-3 | Пишет код, но не видит архитектуры |
| Middle | 4-5 | Пишет хороший код, но не строит систему |
| Senior | 6-7 | Строит систему, выбирает стек, ревьюит |
| Lead | 8-9 | Ведёт направление, учит других |
| Architect | 10 | Видит всю картинку кросс-проектно |
