# Установка

## Linux

### Из GitHub Releases

Скачать последний бинарник со [страницы релизов](https://github.com/RedAlexDad/audiosub/releases):

```bash
curl -sL https://github.com/RedAlexDad/audiosub/releases/latest/download/audiosub -o audiosub
chmod +x audiosub
sudo mv audiosub /usr/local/bin/  # опционально
```

**Требования:**
- PulseAudio (обычно уже установлен)
- libpulse0 — клиентская библиотека

**Захват системного аудио** — работает из коробки через PulseAudio monitor.

### Vosk SDK (опционально)

Если нужен движок Vosk:

```bash
curl -sL -o /tmp/vosk.zip \
  https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-x86_64-0.3.45.zip
unzip -q /tmp/vosk.zip -d /tmp/vosk
sudo cp /tmp/vosk/libvosk.so /usr/local/lib/
sudo ldconfig
```

---

## Windows

### Предварительная сборка (mingw)

Собрать бинарник можно через WSL или кросс-компиляцией:

```bash
# Установить mingw-w64 (на Linux)
sudo apt install mingw-w64

# Собрать
cargo build --release --target x86_64-pc-windows-gnu
# Бинарник: target/x86_64-pc-windows-gnu/release/audiosub.exe
```

**Или** собрать на самой Windows:

```powershell
# Установить Rust: https://rustup.rs
# Установить MSVC Build Tools: https://visualstudio.microsoft.com/downloads/

git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub
cargo build --release
# Бинарник: target/release/audiosub.exe
```

### Захват аудио

На Windows audiosub использует **микрофон** (через CPAL/WASAPI).

Для захвата **системного аудио** (звук из браузера, плеера и т.д.) требуется виртуальный аудиокабель:

1. **VB-Cable** (бесплатно): https://vb-audio.com/Cable/
2. **Установить**, выбрать как устройство вывода в настройках звука Windows
3. Запустить audiosub с указанием устройства `CABLE Output`

### Модели

```powershell
# Скачать модель
audiosub.exe --download-model whisper:tiny

# Запуск
audiosub.exe
```

---

## macOS

### Сборка

```bash
# Установить Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/RedAlexDad/audiosub.git
cd audiosub
cargo build --release
# Бинарник: target/release/audiosub
```

### Захват аудио

На macOS audiosub использует **микрофон** (через CPAL/CoreAudio).

Для захвата **системного аудио** нужно установить виртуальный драйвер:

1. **BlackHole** (бесплатно): https://github.com/ExistentialAudio/BlackHole
2. Создать Multi-Output Device в Audio MIDI Setup
3. Направить системный звук на BlackHole
4. Запустить audiosub с устройством `BlackHole`

### Модели

```bash
./audiosub --download-model whisper:tiny
./audiosub
```

---

## Docker

См. [docker.md](docker.md).

## Сборка из исходников

См. [build.md](build.md).
