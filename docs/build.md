# Сборка из исходников

## Зависимости

```bash
# Rust (см. rust-toolchain.toml для версии)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Системные пакеты
sudo apt install libpulse-dev cmake clang
```

## Сборка

```bash
# Клонировать
git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub

# Собрать (release, whisper + vosk runtime)
make release
# Бинарник: release/audiosub

# Или напрямую через cargo
cargo build --release
```

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
