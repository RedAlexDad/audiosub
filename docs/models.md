# Модели

audiosub поддерживает два движка ASR, каждый со своим форматом моделей.

## Авто-детект

При запуске audiosub ищет файлы моделей в:

1. **Директория бинарника** (где лежит `audiosub`)
2. **Текущая рабочая директория**

Поддерживаемые типы файлов:
- `*.gguf`, `*.bin`, `*.ggml` — модели Whisper
- `*vosk*` (директория) — модель Vosk

Приоритет: **Vosk директория > первый по алфавиту** (например `ggml-base.bin` выберется раньше `ggml-tiny.bin`).

## Скачать модели

```bash
# Авто-выбор (Vosk если есть libvosk.so, иначе Whisper tiny)
audiosub --download-model

# Vosk (русский, ~42 MB)
audiosub --download-model vosk
audiosub --download-model vosk:small-ru

# Whisper варианты (скачиваются с HuggingFace)
audiosub --download-model whisper:tiny     # ~75 MB
audiosub --download-model whisper:base     # ~150 MB
audiosub --download-model whisper:small    # ~500 MB
audiosub --download-model whisper:medium   # ~1.5 GB
audiosub --download-model whisper:large    # ~3 GB (large-v3)
audiosub --download-model whisper:turbo    # ~1.5 GB (быстрый)
```

Модели скачиваются в **текущую директорию**.

> **Примечание:** `whisper:large` теперь качает `ggml-large-v3.bin` —
> оригинальный `ggml-large.bin` (v1) удалён с HuggingFace.

## Ручная установка

### Whisper

Скачать с [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp/tree/main):

```bash
# Внимание: в некоторых сетях hf.co виснет на HTTP/2 — используйте -4 --http1.1
curl -4 --http1.1 -sL -o ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

Таблица моделей Whisper:

| Модель | Файл | Размер | Качество | Скорость |
|--------|------|--------|----------|----------|
| tiny | ggml-tiny.bin | ~75 MB | низкое | быстрая |
| base | ggml-base.bin | ~150 MB | низкое | быстрая |
| small | ggml-small.bin | ~500 MB | среднее | средняя |
| medium | ggml-medium.bin | ~1.5 GB | высокое | медленная |
| large | ggml-large-v3.bin | ~3 GB | высочайшее | медленная |
| turbo | ggml-large-v3-turbo.bin | ~1.5 GB | высокое (large-v3) | быстрая |

### Vosk

Скачать с [alphacephei.com](https://alphacephei.com/vosk/models):

```bash
curl -sL -o /tmp/vosk.zip \
  https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip
unzip -q /tmp/vosk.zip -d /path/to/models
```

Все доступные модели: [alphacephei.com/vosk/models](https://alphacephei.com/vosk/models)

## Раздельные пути для движков

Можно указать разные пути для каждого движка в `audiosub.toml`:

```toml
[asr]
model_path_vosk = "models/vosk-model-small-ru-0.22"
model_path_whisper = "models/ggml-base.bin"
```

Нужный путь выбирается автоматически по активному движку.

## Как работает поиск модели

Цепочка приоритетов:

1. `--model` (CLI флаг)
2. `model_path_vosk` / `model_path_whisper` в конфиге
3. `model_path` в конфиге (общий)
4. Авто-детект (директория бинарника → CWD)
5. `~/.cache/audiosub/models/vosk-model-small-en-us-0.15` (жёсткий дефолт)
6. Ошибка с предложением скачать модель (`--download-model`)
