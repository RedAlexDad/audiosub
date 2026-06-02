# Usage

## TUI mode (default)

```bash
audiosub
```

Starts the terminal interface with:

- **Header**: engine name + model, status, segment count, elapsed time, VU meter
- **Recognition tab** (default): real-time partial transcription
- **Segments tab**: finalized subtitle segments
- **Logs tab**: debug logs

### TUI controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Tab` / `Shift+Tab` | Next/previous screen |
| `p` | Pause/resume recognition |
| `r` | Reset session |
| `s` | Export SRT |
| `S` | Export TXT |
| `↑` / `↓` | Scroll segments |
| `PgUp` / `PgDown` | Page scroll |
| `Home` / `End` | Top/bottom |
| `c` | Clear segments (Segments tab) |
| `R` | Refresh logs (Logs tab) |

## CLI mode

```bash
audiosub --no-tui --duration 60
```

| Flag | Description |
|------|-------------|
| `-d, --duration <sec>` | Recording duration (default: unlimited) |
| `--no-tui` | Disable TUI, plain console mode |
| `-o, --output <path>` | Output subtitle file |
| `--format <srt\|vtt>` | Subtitle format (default: srt) |
| `--list-devices` | List available audio sources |
| `-m, --model <path>` | Override model path |
| `--engine <vosk\|whisper>` | Select ASR engine |

## Engine selection

Priority (highest to lowest):

1. `--engine` CLI flag
2. `AUDIOSUB_ENGINE` environment variable
3. `engine` in `audiosub.toml`
4. Auto-detection based on available model files

### Example

```bash
# Explicit engine
audiosub --engine vosk
audiosub --engine whisper

# Env var
AUDIOSUB_ENGINE=whisper audiosub
```

## Configuration

Optional `audiosub.toml` in the current directory or `~/.cache/audiosub/`:

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

Without a config file, defaults are auto-detected:
- **Audio device**: PulseAudio default monitor source
- **Engine**: based on available model files
- **Model path**: auto-scanned in binary directory and CWD

## Audio device detection

```bash
# List available monitor sources
audiosub --list-devices

# Explicit device
audiosub --device alsa_output.pci-0000_00_1f.3.analog-stereo.monitor
```

See [models.md](models.md) for model management.
