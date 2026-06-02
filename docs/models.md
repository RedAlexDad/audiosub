# Models

audiosub supports two ASR engines, each with its own model format.

## Auto-detection

On startup, audiosub scans for model files in:

1. The **binary's directory** (where `audiosub` is located)
2. The **current working directory**

Supported file patterns:
- `*.gguf`, `*.bin`, `*.ggml` — Whisper models
- `*vosk*` directory — Vosk model directory

Priority: **Vosk model directory > first alphabetically** (so `ggml-base.bin` is preferred over `ggml-tiny.bin`).

## Download models

```bash
# Auto-select (Vosk if libvosk.so present, else Whisper tiny)
audiosub --download-model

# Vosk (Russian, ~42 MB)
audiosub --download-model vosk
audiosub --download-model vosk:small-ru

# Whisper variants (download from HuggingFace)
audiosub --download-model whisper:tiny     # ~75 MB
audiosub --download-model whisper:base     # ~150 MB
audiosub --download-model whisper:small    # ~500 MB
audiosub --download-model whisper:medium   # ~1.5 GB
audiosub --download-model whisper:large    # ~3 GB
audiosub --download-model whisper:turbo    # ~1.5 GB (fast)
```

Models are downloaded to the **current directory**.

## Manual model setup

### Whisper

Download from [HuggingFace 🤗](https://huggingface.co/ggerganov/whisper.cpp/tree/main):

```bash
curl -sL -o ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

Whisper model files:
| Model | Size | Quality | Speed |
|-------|------|---------|-------|
| tiny | ~75 MB | lowest | fastest |
| base | ~150 MB | low | fast |
| small | ~500 MB | medium | medium |
| medium | ~1.5 GB | high | slow |
| large | ~3 GB | highest | slowest |
| turbo | ~1.5 GB | high (large-v3) | fast |

### Vosk

Download from [alphacephei.com](https://alphacephei.com/vosk/models):

```bash
curl -sL -o /tmp/vosk.zip \
  https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip
unzip -q /tmp/vosk.zip -d /path/to/models
```

Vosk models available at [alphacephei.com/vosk/models](https://alphacephei.com/vosk/models).

## Engine-specific config

Set different model paths for each engine in `audiosub.toml`:

```toml
[asr]
model_path_vosk = "models/vosk-model-small-ru-0.22"
model_path_whisper = "models/ggml-base.bin"
```

The correct path is selected automatically based on the active engine.

## How model resolution works

Priority chain:

1. `--model` CLI flag
2. `model_path_vosk` / `model_path_whisper` in config
3. `model_path` in config (fallback)
4. Auto-detection (binary dir → CWD)
5. `~/.cache/audiosub/models/vosk-model-small-en-us-0.15` (hard default)
6. Error with download suggestion
