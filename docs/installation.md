# Установка

## Из GitHub Releases (рекомендуется)

Скачать последний бинарник со [страницы релизов](https://github.com/RedAlexDad/audiosub/releases):

```bash
# Скачать
curl -sL https://github.com/RedAlexDad/audiosub/releases/latest/download/audiosub -o audiosub
chmod +x audiosub

# (Опционально) установить системно
sudo mv audiosub /usr/local/bin/
```

Бинарник включает **оба движка**: Whisper встроен, Vosk загружается runtime если есть `libvosk.so`.

### Требования

- **Linux** (x86_64)
- **PulseAudio** — для захвата аудио (обычно уже установлен)
- **libpulse0** — клиентская библиотека PulseAudio

### Опционально: Vosk SDK

Для использования движка Vosk:

```bash
curl -sL -o /tmp/vosk.zip \
  https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-x86_64-0.3.45.zip
unzip -q /tmp/vosk.zip -d /tmp/vosk
sudo cp /tmp/vosk/libvosk.so /usr/local/lib/
sudo ldconfig
```

После этого `audiosub --engine vosk` будет работать.

## Через Docker

См. [docker.md](docker.md).

## Сборка из исходников

См. [build.md](build.md).
