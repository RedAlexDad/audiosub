# План многопоточной архитектуры

## Проблема

Текущая архитектура — один поток, всё последовательно:

```
loop {
    terminal.draw()             // ~5ms
    handle_input()              // ~0ms (non-blocking poll)
    capture.read(read_step)     // ~25ms блокировка в pa_simple_read
    // ресемплер может вернуть None — пропускает всё ниже
    update_audio_levels()       // только когда ресемплер выдал выход
    engine.feed_audio()
    engine.partial_text()
    engine.drain_segments()
}
```

Всё последовательно, одно блокирует другое. Ресемплер (`rubato::Fft`) копит входные сэмплы, пока не наберёт `input_needed ≈ 1024`, поэтому `capture.read()` возвращает `None` большую часть времени. VU-метр обновляется рывками, UI тормозит.

## Предложение: 3 потока

```
┌──────────────────┐    raw f32    ┌─────────────────┐   UiUpdate    ┌─────────────────┐
│  Capture Thread  │ ────────────→ │   ASR Thread    │ ────────────→ │   TUI Thread    │
│                  │               │                 │               │                 │
│  pa_simple_read  │ mpsc::channel │  resample()     │ mpsc::channel │  poll input     │
│  send(raw_f32)   │ <Vec<f32>>    │  engine.*()     │ <UiUpdate>    │  recv results   │
│                  │               │  compute_levels │               │  draw() ~60fps  │
│                  │               │  send(UiUpdate) │               │  sleep(5ms)     │
└────────┬─────────┘               └────────┬────────┘               └────────┬────────┘
         │                                  │                                 │
         └─────── Arc<AtomicBool> (stop) ───┴─────────────────────────────────┘
```

### Поток 1: Capture

Только `pa_simple_read()`, отправляет сырые f32 сэмплы.

```rust
fn capture_thread(
    mut capture: PulseCapture,
    tx: mpsc::Sender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let chunk = 400; // ~9ms блокировки — низкая задержка при выходе
    while !stop.load(Ordering::Relaxed) {
        let buf = capture.read_raw(chunk)?;
        if tx.send(buf).is_err() {
            break; // приёмник отпал = TUI закрылся
        }
    }
    Ok(())
}
```

- **PulseCapture**: нужно `unsafe impl Send`, потому что `Simple` оборачивает `*mut pa_simple`. Безопасно на практике — потоr используется исключительно в одном потоке (pa_simple thread-safe внутри, но мы не обращаемся из нескольких потоков).
- **Stop latency**: максимум ~9ms (один chunk), затем проверка `AtomicBool`.

### Поток 2: ASR

Принимает сырое аудио, ресемплит, кормит engine, забирает результаты, считает уровни.

```rust
struct UiUpdate {
    partial: String,
    segments: Vec<Segment>,
    rms: f32,
    peak: f32,
    sample_count: usize,
}

fn asr_thread(
    mut engine: Box<dyn AsrEngine>,
    mut buffer: SubtitleBuffer,
    mut output: SubtitleOutput,
    rx: mpsc::Receiver<Vec<f32>>,
    tx: mpsc::Sender<UiUpdate>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let mut resampler = AudioResampler::new(source_rate, target_rate)?;
    let mut level_tracker = LevelTracker::new();

    while !stop.load(Ordering::Relaxed) {
        match rx.recv() {
            Ok(raw) => {
                if let Some(resampled) = resampler.process(&raw)? {
                    level_tracker.update(&raw);
                    engine.feed_audio(&resampled)?;
                    let partial = engine.partial_text()?;
                    let segments = engine.drain_segments()?;

                    for seg in segments {
                        buffer.push(seg);
                    }
                    while let Some(ready) = buffer.flush() {
                        output.append(&ready)?;
                    }

                    let _ = tx.send(UiUpdate {
                        partial,
                        segments: buffer.pending(),
                        rms: level_tracker.rms(),
                        peak: level_tracker.peak(),
                        sample_count: resampled.len(),
                    });
                }
            }
            Err(_) => break, // capture thread умер
        }
    }

    // Финализация: слить остатки
    let final_segments = engine.finalize()?;
    for seg in final_segments { buffer.push(seg); }
    while let Some(ready) = buffer.flush() { output.append(&ready)?; }
    Ok(())
}
```

- **ASR thread владеет**: engine, buffer, output, resampler — все требуют `&mut self`, никогда не шарятся.
- **Блокировка**: `rx.recv()` — ждёт новое аудио от capture. Нет busy-loop.
- **Send для UiUpdate**: `Segment` содержит `String + Duration + ...`, по умолчанию `Send`.
- **Lossy канал**: ASR→TUI — unbounded. TUI дренирует через `try_recv()` в цикле, оставляя только последнее сообщение.

### Поток 3: TUI

Poll input, recv результатов от ASR, рисует UI.

```rust
fn tui_thread(
    app: &mut TuiApp,
    terminal: &mut Terminal<impl Backend>,
    rx: mpsc::Receiver<UiUpdate>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    while app.running {
        // Дренируем все обновления, оставляем последнее
        while let Ok(update) = rx.try_recv() {
            app.update_audio(update.sample_count);
            app.update_audio_levels_from(update.rms, update.peak);
            app.set_partial(&update.partial);
            app.set_segments(update.segments);
        }

        // Ввод (non-blocking)
        if handle_input(app)? {
            stop.store(true, Ordering::Relaxed);
            break;
        }

        terminal.draw(|f| view::render(app, f))?;
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
```

- **Никогда не блокируется на audio**: `try_recv()` — non-blocking. Задержка audio влияет только на скорость появления новых результатов, не на скорость UI-цикла.
- **Ввод**: `event::poll(0)` — как сейчас, non-blocking.
- **Draw**: каждые ~5ms. Плавная анимация.
- **Выход**: `stop = true`, break — потоки 1 и 2 останавливаются на следующей итерации.

## Edge cases

### Quit — capture thread висит в pa_simple_read

Capture thread читает чанками по ~400 сэмплов (~9ms). Максимальное ожидание: ~9ms, затем проверка `stop`.

**Можно ещё проще**: `std::thread::spawn` для capture thread, не джойнить его при выходе. При завершении процесса ОС подчистит. Никакой координации не нужно.

**Рекомендация**: так и сделать. Capture thread не джойнится — просто бросаем.

### Backlog в ASR→Capture канале

Если ASR медленнее реального времени (большая Vosk модель на слабом CPU), канал между capture и ASR растёт бесконечно. Это ОК — capture продолжает читать, не теряя аудио. UI показывает результаты с задержкой.

Можно ограничить: `sync_channel(100)`. Тогда при переполнении capture сбрасывает старые данные. Но это потеря аудио — unbounded предпочтительнее.

### Переполнение ASR→TUI канала

Если TUI медленнее ASR (маловероятно), сообщения копятся. `try_recv()` в цикле гарантирует, что TUI видит последнее состояние. Старые сообщения просто дропаются.

## План реализации

### Файлы для изменений:

1. **`src/audio/pulse.rs`** — добавить `unsafe impl Send for PulseCapture {}` с комментарием про thread-safety pa_simple.
2. **`src/audio/pulse.rs`** — добавить `read_raw(&mut self, n: usize) -> Result<Vec<f32>>`, возвращает сырые (нересемпленные) f32.
3. **`src/tui/capture.rs`** — полный рефакторинг: 3 thread spawn + каналы вместо одного цикла.
4. **`src/tui/input.rs`** — изменений не нужно (уже декомпозирован).
5. **`src/tui/app.rs`** — добавить `update_audio_levels_from(rms, peak)` для прямой установки уровня без Vec.

### Новые структуры:

- `UiUpdate` — либо в `src/tui/capture.rs`, либо отдельным модулем.

### Типы каналов:

- Capture→ASR: `mpsc::channel::<Vec<f32>>()` (unbounded)
- ASR→TUI: `mpsc::channel::<UiUpdate>()` (unbounded)
- Stop: `Arc<AtomicBool>` общий для всех трёх потоков

## Альтернатива: 2 потока

Склеить Capture и ASR в один «audio pipeline» поток:

```
[Audio Pipeline] ──UiUpdate──→ [TUI]
```

Плюс: проще (1 канал, 2 потока). Минус: capture блокируется, если ASR медленный (они в одном потоке). Но на практике разница мала — `pa_simple_read` и так блокирующий, а обработка ASR быстрая (<1ms).

**Рекомендация**: начать с 2 потоков (Audio Pipeline + TUI). Это проще и даёт главный профит (TUI никогда не блокируется). Если профилирование покажет, что Capture и ASR стоит разделить — апгрейднуть до 3.
