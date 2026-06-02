# Docker

audiosub is available as a Docker image on [GitHub Container Registry](https://github.com/RedAlexDad/audiosub/pkgs/container/audiosub).

## Quick start

```bash
# Run with Whisper
docker run --rm ghcr.io/redalexdad/audiosub:latest --help

# Run with Vosk (includes libvosk.so in the image)
docker run --rm ghcr.io/redalexdad/audiosub:latest --engine vosk --help
```

## TUI mode

For interactive TUI mode, you need PulseAudio passthrough:

```bash
docker run --rm -it \
  --device /dev/snd \
  -e PULSE_SERVER=unix:/tmp/pulse/pulseaudio.socket \
  -v /run/user/$(id -u)/pulse:/tmp/pulse \
  ghcr.io/redalexdad/audiosub:latest
```

## CLI mode with audio capture

```bash
docker run --rm \
  --group-add audio \
  -v /dev/snd:/dev/snd \
  ghcr.io/redalexdad/audiosub:latest \
  --no-tui --duration 30
```

## Building locally

```bash
# Build with Vosk SDK (runtime) + Whisper
docker build --build-arg ENGINE=vosk -t audiosub .

# Build without Vosk (smaller image)
docker build --build-arg ENGINE=whisper -t audiosub .

# Run
docker run --rm audiosub --help
```

## Image variants

The Docker image supports three `ENGINE` build args:

| ENGINE | libvosk.so | Whisper | Binary size |
|--------|-----------|---------|-------------|
| `whisper` | No | Yes | ~50 MB |
| `vosk` | Yes | Yes | ~60 MB |
| `both` | Yes | Yes | ~60 MB |

The default is `vosk` (both engines available at runtime).
