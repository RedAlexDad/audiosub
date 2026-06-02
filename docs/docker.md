# Docker

Образ audiosub доступен на [GitHub Container Registry](https://github.com/RedAlexDad/audiosub/pkgs/container/audiosub).

## Быстрый старт

```bash
# Запуск с Whisper
docker run --rm ghcr.io/redalexdad/audiosub:latest --help

# Запуск с Vosk (libvosk.so уже установлен в образе)
docker run --rm ghcr.io/redalexdad/audiosub:latest --engine vosk --help
```

## TUI режим

Для интерактивного TUI нужен PulseAudio:

```bash
docker run --rm -it \
  --device /dev/snd \
  -e PULSE_SERVER=unix:/tmp/pulse/pulseaudio.socket \
  -v /run/user/$(id -u)/pulse:/tmp/pulse \
  ghcr.io/redalexdad/audiosub:latest
```

## CLI с захватом аудио

```bash
docker run --rm \
  --group-add audio \
  -v /dev/snd:/dev/snd \
  ghcr.io/redalexdad/audiosub:latest \
  --no-tui --duration 30
```

## Сборка локально

```bash
# С Vosk SDK (runtime) + Whisper
docker build --build-arg ENGINE=vosk -t audiosub .

# Без Vosk (меньший образ)
docker build --build-arg ENGINE=whisper -t audiosub .

# Запуск
docker run --rm audiosub --help
```

## Варианты образа

Поддерживаемые `ENGINE` аргументы:

| ENGINE | libvosk.so | Whisper | Размер |
|--------|-----------|---------|--------|
| `whisper` | Нет | Да | ~50 MB |
| `vosk` | Да | Да | ~60 MB |
| `both` | Да | Да | ~60 MB |

По умолчанию `vosk` (оба движка доступны в runtime).
