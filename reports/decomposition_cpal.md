# План декомпозиции: кросс-платформенное аудио через CPAL

## Мотивация
Сейчас audiosub работает только на Linux через PulseAudio. Для поддержки Windows и macOS нужен кросс-платформенный аудио-бэкенд.

## Подход

**CPAL** (Cross-Platform Audio Library) — основная Rust-библиотека для аудиоввода.
- Linux: ALSA (замена PulseAudio)
- Windows: WASAPI
- macOS: CoreAudio

CPAL не поддерживает loopback (захват системного аудио) напрямую. На Windows нужен `wasapi` с loopback флагом, на macOS — виртуальный драйвер (BlackHole). Поэтому:

1. **Linux** — оставляем PulseCapture (loopback работает)
2. **Windows/macOS** — CpalCapture (микрофон, loopback позже)

---

## Задача 1: Добавить CPAL зависимость

**Cargo.toml:**
```toml
cpal = "0.15"
```

CPAL работает на всех платформах, без feature-флагов.

---

## Задача 2: Создать CpalCapture (src/audio/cpal.rs)

Новый тип, реализующий `AudioCapture`:

```rust
pub struct CpalCapture {
    device_name: String,
    source_rate: u32,
    target_rate: u32,
    stream: Option<cpal::Stream>,
    resampler: Option<AudioResampler>,
    rx: Option<mpsc::Receiver<Vec<f32>>>,
    tx: Option<mpsc::Sender<Vec<f32>>>,
    sample_rate_actual: Arc<AtomicU32>,
    buffer: Vec<f32>,           // для накопления неполных блоков
}
```

**`new(device, source_rate)`** — сохраняет параметры.

**`start()`**:
1. Выбрать устройство: `device_name` или `cpal::default_input_device()`
2. Выбрать поддерживаемую конфигурацию (sample rate, channels, sample format)
3. Создать канал `mpsc::channel::<Vec<f32>>`
4. Создать `cpal::Stream` с callback: получает данные, конвертирует в `Vec<f32>`, отправляет в канал
5. Создать `AudioResampler` для ресемплинга

**`read(chunk_size)`**:
1. Читать из `rx` канала, пока не наберётся `chunk_size` сэмплов
2. Пропустить через ресемплер
3. Вернуть `AudioChunk`

**`stop()`**:
1. Drop stream (автоматически останавливается)
2. Закрыть канал

**`sample_rate()`** — возвращает `target_rate` (16000).

**`read_raw(n)`** — как в PulseCapture, для TUI (возвращает без ресемплинга).

---

## Задача 3: Обновить audio/mod.rs

```rust
// Платформо-зависимый выбор
#[cfg(target_os = "linux")]
pub use pulse::PulseCapture as DefaultCapture;

#[cfg(not(target_os = "linux"))]
pub use cpal::CpalCapture as DefaultCapture;

pub type DefaultCapture = PulseCapture;  // Linux
// или
pub type DefaultCapture = CpalCapture;   // Windows/macOS
```

Добавить `pub mod cpal;`.

---

## Задача 4: Обновить monitor.rs — кроссплатформенный автодетект

**Linux**: `find_default_monitor()` через `pactl` (как сейчас).
**Windows/macOS**: через CPAL:

```rust
pub fn list_sources() -> Result<Vec<String>> {
    cpal::devices()
        .map(|d| d.name())
        .collect()
}
```

---

## Задача 5: Обновить main.rs и session/mod.rs

Заменить `PulseCapture::new(...)` на `DefaultCapture::new(...)` или выбирать по конфигу/платформе.

---

## Задача 6: Сборка release-win

`make release-win` — сейчас использует PulseAudio, упадёт. После CPAL будет работать.

Нужен кросс-компилятор mingw-w64 для Windows:
```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## Приоритет и порядок

1. **CPAL зависимость + CpalCapture** (ядро — можно тестировать на Linux/macOS)
2. **audio/mod.rs — тип по умолчанию для платформы** (интеграция)
3. **Устройства — кроссплатформенный список** (list_devices)
4. **Сборка release-win** (проверка на Windows)

---

## Файлы для изменений

| Файл | Изменения |
|------|-----------|
| `Cargo.toml` | + `cpal = "0.15"` |
| `src/audio/cpal.rs` | Новый: `CpalCapture` struct + `AudioCapture` impl |
| `src/audio/mod.rs` | + `mod cpal`, платформенный `DefaultCapture` |
| `src/audio/monitor.rs` | + кроссплатформенный `list_sources()` |
| `src/main.rs` | Использовать `DefaultCapture` вместо `PulseCapture` |
| `src/session/mod.rs` | Использовать `DefaultCapture` вместо `PulseCapture` |
| `mk/release.mk` | Актуализировать `release-win` |
