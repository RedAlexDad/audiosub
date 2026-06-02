# Installation

## From GitHub Releases (recommended)

Download the latest binary from the [releases page](https://github.com/RedAlexDad/audiosub/releases):

```bash
# Download
curl -sL https://github.com/RedAlexDad/audiosub/releases/latest/download/audiosub -o audiosub
chmod +x audiosub

# (Optional) install system-wide
sudo mv audiosub /usr/local/bin/
```

This binary includes **both engines**: Whisper is built-in, Vosk is loaded at runtime if `libvosk.so` is present.

### System requirements

- **Linux** (x86_64)
- **PulseAudio** — for audio capture (usually pre-installed on desktop Linux)
- **libpulse0** — PulseAudio client library (pre-installed on most distros)

### Optional: Vosk SDK

If you want to use the Vosk engine, download and install `libvosk.so`:

```bash
curl -sL -o /tmp/vosk.zip \
  https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-x86_64-0.3.45.zip
unzip -q /tmp/vosk.zip -d /tmp/vosk
sudo cp /tmp/vosk/libvosk.so /usr/local/lib/
sudo ldconfig
```

Then `audiosub --engine vosk` will work.

## Via Docker

See [docker.md](docker.md).

## Building from source

See [build.md](build.md).
