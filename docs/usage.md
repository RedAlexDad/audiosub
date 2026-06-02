# Использование

## TUI (по умолчанию)

```bash
audiosub
```

Интерфейс:

- **Шапка**: имя движка + модель, статус, счётчик сегментов, время, VU-метр
- **Recognition** (по умолчанию): распознавание в реальном времени
- **Segments** : готовые сегменты субтитров
- **Logs** : отладочные логи

### Управление в TUI

| Клавиша | Действие |
|---------|----------|
| `q` / `Esc` | Выход |
| `Tab` / `Shift+Tab` | Следующий/предыдущий экран |
| `p` | Пауза/продолжить |
| `r` | Сброс сессии |
| `s` | Экспорт SRT |
| `S` | Экспорт TXT |
| `↑` / `↓` | Скролл сегментов |
| `PgUp` / `PgDown` | Скролл страницами |
| `Home` / `End` | Начало/конец |
| `c` | Очистить сегменты (Segments) |
| `R` | Обновить логи (Logs) |

## CLI режим

```bash
audiosub --no-tui --duration 60
```

| Флаг | Описание |
|------|----------|
| `-d, --duration <сек>` | Длительность записи (по умолчанию: безлимит) |
| `--no-tui` | Без TUI, консольный режим |
| `-o, --output <путь>` | Файл для субтитров |
| `--format <srt\|vtt>` | Формат субтитров |
| `--list-devices` | Список аудиоисточников |
| `-m, --model <путь>` | Путь к модели |
| `--engine <vosk\|whisper>` | Выбор движка ASR |

## Выбор движка

Приоритет (от высшего к низшему):

1. `--engine` (CLI флаг)
2. `AUDIOSUB_ENGINE` (переменная окружения)
3. `engine` в `audiosub.toml`
4. Авто-детект по найденным файлам моделей

### Примеры

```bash
audiosub --engine vosk
audiosub --engine whisper
AUDIOSUB_ENGINE=whisper audiosub
```

## Конфигурация

Опциональный `audiosub.toml` в текущей директории или `~/.cache/audiosub/`:

```toml
[audio]
device = "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
sample_rate = 16000
channels = 1

[asr]
engine = "whisper"
model_path = "models/ggml-base.bin"
model_path_vosk = "models/vosk-model-small-ru-0.22"
model_path_whisper = "models/ggml-base.bin"
lang = "ru-RU"

[subtitle]
format = "srt"
output = "subtitles.srt"
buffer_ms = 2000
max_duration_ms = 5000
```

Без конфига используются авто-детект:
- **Аудиоустройство**: монитор PulseAudio по умолчанию
- **Движок**: по типу найденной модели
- **Модель**: сканируется директория бинарника и CWD

## Аудиоустройства

```bash
# Список доступных источников
audiosub --list-devices

# Явное указание устройства
audiosub --device alsa_output.pci-0000_00_1f.3.analog-stereo.monitor
```

Подробнее про модели — в [models.md](models.md).
