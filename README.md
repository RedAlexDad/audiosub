# audiosub

Автоматические субтитры в реальном времени для Linux. Захватывает аудио системы и генерирует субтитры через **Whisper** (встроен) или **Vosk** (runtime).

## Быстрый старт

```bash
# Скачать бинарник и модель
curl -sL https://github.com/RedAlexDad/audiosub/releases/latest/download/audiosub -o audiosub
chmod +x audiosub
./audiosub --download-model whisper:tiny

# Запуск — сам находит модель и устройство
./audiosub
```

## Возможности

- **Два движка ASR**: Whisper (встроен, портативный) и Vosk (runtime через libvosk.so)
- **Кросс-платформенность**: Linux (PulseAudio), Windows (WASAPI), macOS (CoreAudio)
- **Без конфига**: автодетект модели, аудиоустройства и движка
- **TUI**: интерактивный терминальный интерфейс
- **CLI**: консольный режим для пайплайнов (`--no-tui`)
- **Субтитры**: экспорт в SRT и VTT
- **Docker**: готовые образы на [ghcr.io](https://github.com/RedAlexDad/audiosub/pkgs/container/audiosub)
- **Авто-загрузка моделей**: `--download-model`

## Документация

| Раздел | Ссылка |
|--------|--------|
| Установка | [docs/installation.md](docs/installation.md) |
| Использование | [docs/usage.md](docs/usage.md) |
| Модели | [docs/models.md](docs/models.md) |
| Docker | [docs/docker.md](docs/docker.md) |
| Сборка из исходников | [docs/build.md](docs/build.md) |

## Быстрая справка

```bash
audiosub                        # TUI (whisper, авто-модель)
audiosub --engine vosk          # Vosk engine
audiosub --no-tui --duration 30 # CLI, 30 секунд
audiosub --list-devices         # Список аудиоисточников
audiosub --download-model       # Скачать модель (авто)
audiosub --download-model whisper:base  # Конкретная модель
audiosub --help                 # Полная справка
```

## Конфиг (опционально)

`audiosub.toml` в текущей директории:

```toml
[audio]
device = "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
sample_rate = 16000

[asr]
engine = "whisper"
model_path_whisper = "ggml-base.bin"
model_path_vosk = "vosk-model-small-ru-0.22"
```

Подробнее в [docs/usage.md#конфигурация](docs/usage.md#конфигурация).

## Лицензия

MIT
