# audiosub

Real-time automatic subtitles for Linux. Captures system audio and generates subtitles via **Whisper** (built-in) or **Vosk** (runtime).

## Quick start

```bash
# Download the binary and a model
curl -sL https://github.com/RedAlexDad/audiosub/releases/latest/download/audiosub -o audiosub
chmod +x audiosub
./audiosub --download-model whisper:tiny

# Run — it auto-detects the model and starts
./audiosub
```

## Features

- **Two ASR engines**: Whisper (built-in, portable) and Vosk (runtime via libvosk.so)
- **No config required**: auto-detects model files, audio device, and engine
- **TUI mode**: interactive terminal interface with real-time transcription
- **CLI mode**: headless operation for pipelines (`--no-tui`)
- **Subtitle export**: SRT and VTT formats
- **Docker**: ready-to-use images on [ghcr.io](https://github.com/RedAlexDad/audiosub/pkgs/container/audiosub)
- **Auto-download models**: `--download-model` fetches models automatically

## Documentation

| Topic | Link |
|-------|------|
| Installation | [docs/installation.md](docs/installation.md) |
| Usage (CLI + TUI) | [docs/usage.md](docs/usage.md) |
| Models | [docs/models.md](docs/models.md) |
| Docker | [docs/docker.md](docs/docker.md) |
| Building from source | [docs/build.md](docs/build.md) |

## Quick reference

```bash
audiosub                        # TUI mode (whisper, auto model)
audiosub --engine vosk          # Use Vosk engine
audiosub --no-tui --duration 30 # CLI mode, 30 seconds
audiosub --list-devices         # Show audio sources
audiosub --download-model       # Download model (auto)
audiosub --download-model whisper:base  # Specific model
audiosub --help                 # Full help
```

## Configuration

Optional `audiosub.toml` in the current directory:

```toml
[audio]
device = "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
sample_rate = 16000

[asr]
engine = "whisper"
model_path_whisper = "ggml-base.bin"
model_path_vosk = "vosk-model-small-ru-0.22"
```

See [docs/usage.md#configuration](docs/usage.md#configuration) for details.

## License

MIT
