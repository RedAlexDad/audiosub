use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

use crate::asr::{AsrEngine, Segment};
use crate::audio::{AudioResampler, PulseCapture};
use crate::subtitle::{SubtitleBuffer, SubtitleOutput};
use crate::tui::{input, view};
use crate::tui::app::TuiApp;
use crate::tui::screen::Screen;

// ── Messages ──

/// Raw audio + levels sent from Capture thread to ASR thread.
pub struct AudioData {
    pub samples: Vec<f32>,
    pub rms: f32,
    pub peak: f32,
}

/// UI state update sent from ASR thread to TUI thread.
pub struct UiUpdate {
    pub partial: String,
    pub segments: Vec<Segment>,
    pub rms: f32,
    pub peak: f32,
    pub sample_count: usize,
}

// ── Level helpers ──

fn compute_rms(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = data.iter().map(|&s| s * s).sum();
    (sum_sq / data.len() as f32).sqrt().min(1.0)
}

fn compute_peak(data: &[f32]) -> f32 {
    data.iter().map(|&s| s.abs()).fold(0.0_f32, f32::max).min(1.0)
}

// ── Capture thread ──

fn capture_thread(
    capture: &mut PulseCapture,
    tx: mpsc::Sender<AudioData>,
    stop: Arc<AtomicBool>,
    read_chunk: usize,
) -> Result<()> {
    while !stop.load(Ordering::Relaxed) {
        let Some(samples) = capture.read_raw(read_chunk)? else {
            continue;
        };
        let rms = compute_rms(&samples);
        let peak = compute_peak(&samples);
        if tx.send(AudioData { samples, rms, peak }).is_err() {
            break;
        }
    }
    Ok(())
}

// ── ASR thread ──

#[allow(clippy::too_many_arguments)]
fn asr_thread(
    rx: mpsc::Receiver<AudioData>,
    tui_tx: mpsc::Sender<UiUpdate>,
    engine: &mut dyn AsrEngine,
    output: &mut SubtitleOutput,
    buffer: &mut SubtitleBuffer,
    resampler: &mut AudioResampler,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    reset: Arc<AtomicBool>,
    target_rate: u32,
    max_duration_ms: u64,
) -> Result<()> {
    let mut total_resampled = 0usize;

    loop {
        if stop.load(Ordering::Relaxed) && rx.try_recv().is_err() {
            break;
        }

        let audio = match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(a) => a,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        if reset.swap(false, Ordering::Relaxed) {
            engine.reset().ok();
            let _ = buffer.drain();
            resampler.reset();
        }

        if paused.load(Ordering::Relaxed) {
            let _ = tui_tx.send(UiUpdate {
                partial: String::new(),
                segments: vec![],
                rms: audio.rms,
                peak: audio.peak,
                sample_count: 0,
            });
            continue;
        }

        let resampled = resampler.process(&audio.samples)?;
        if resampled.is_empty() {
            continue;
        }

        engine.feed_audio(&resampled)?;

        let partial = engine.partial_text().unwrap_or_default();
        let segments = engine.drain_segments().unwrap_or_default();

        total_resampled += resampled.len();

        for seg in &segments {
            for split in crate::subtitle::split_segment(seg.clone(), max_duration_ms) {
                buffer.push(split);
            }
        }

        let stream_pos_ms = (total_resampled as u64 * 1000) / target_rate as u64;
        for ready in buffer.flush(stream_pos_ms) {
            output.append(&ready)?;
        }

        if tui_tx
            .send(UiUpdate {
                partial,
                segments,
                rms: audio.rms,
                peak: audio.peak,
                sample_count: resampled.len(),
            })
            .is_err()
        {
            break;
        }
    }

    let final_segments = engine.finalize().unwrap_or_default();
    for seg in &final_segments {
        for split in crate::subtitle::split_segment(seg.clone(), max_duration_ms) {
            buffer.push(split);
        }
    }
    resampler.flush().ok();
    for ready in buffer.drain() {
        output.append(&ready)?;
    }
    output.close()?;

    let _ = tui_tx.send(UiUpdate {
        partial: String::new(),
        segments: final_segments,
        rms: 0.0,
        peak: 0.0,
        sample_count: 0,
    });

    Ok(())
}

// ── TUI loop (main thread) ──

fn tui_loop(
    tui_rx: mpsc::Receiver<UiUpdate>,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    reset: Arc<AtomicBool>,
    engine_rate: u32,
    max_duration_ms: u64,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = TuiApp::new(engine_rate, max_duration_ms);

    while app.running {
        terminal.draw(|f| view::render(&mut app, f))?;

        input::handle_input(&mut app)?;

        stop.store(!app.running, Ordering::Relaxed);
        paused.store(app.paused, Ordering::Relaxed);
        if app.reset_requested {
            reset.store(true, Ordering::Relaxed);
            app.reset_requested = false;
        }

        while let Ok(update) = tui_rx.try_recv() {
            app.set_partial(&update.partial);
            if !update.segments.is_empty() {
                app.add_segments(update.segments);
            }
            app.audio_level = Some((update.rms, update.peak));
            if update.sample_count > 0 {
                app.total_samples += update.sample_count;
                app.elapsed =
                    Duration::from_secs_f64(app.total_samples as f64 / app.engine_rate as f64);
            }
        }

        if app.screen == Screen::Logs {
            app.log_needs_refresh = true;
        }

        // ~60fps without busy-wait
        std::thread::sleep(Duration::from_millis(16));
    }

    terminal.draw(|f| view::render(&mut app, f))?;
    std::thread::sleep(Duration::from_secs(1));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

// ── Public entry point ──

pub fn run_tui(
    mut capture: PulseCapture,
    mut engine: Box<dyn AsrEngine>,
    mut output: SubtitleOutput,
    mut buffer: SubtitleBuffer,
    source_rate: u32,
    chunk_size: usize,
    max_duration_ms: u64,
) -> Result<()> {
    let target_rate = 16000;
    let read_chunk = (chunk_size / 4).max(64);

    // Control flags
    let stop = Arc::new(AtomicBool::new(false));
    let paused = Arc::new(AtomicBool::new(false));
    let reset = Arc::new(AtomicBool::new(false));

    // Channels
    let (capture_tx, capture_rx) = mpsc::channel::<AudioData>();
    let (tui_tx, tui_rx) = mpsc::channel::<UiUpdate>();

    // ── Spawn capture thread ──
    let stop_cap = stop.clone();
    let cap_handle = thread::spawn(move || {
        if let Err(e) = capture_thread(&mut capture, capture_tx, stop_cap, read_chunk) {
            tracing::error!("Capture thread: {}", e);
        }
        drop(capture);
    });

    // ── Spawn ASR thread ──
    let stop_asr = stop.clone();
    let paused_asr = paused.clone();
    let reset_asr = reset.clone();
    let mut resampler = AudioResampler::new(source_rate, target_rate)
        .expect("Failed to create resampler for ASR thread");

    let asr_handle = thread::spawn(move || {
        if let Err(e) = asr_thread(
            capture_rx,
            tui_tx,
            &mut *engine,
            &mut output,
            &mut buffer,
            &mut resampler,
            stop_asr,
            paused_asr,
            reset_asr,
            target_rate,
            max_duration_ms,
        ) {
            tracing::error!("ASR thread: {}", e);
        }
        drop(engine);
        drop(output);
        drop(buffer);
        drop(resampler);
    });

    // ── TUI loop on main thread ──
    let result = tui_loop(tui_rx, stop, paused, reset, target_rate, max_duration_ms);

    // Stop workers and wait
    let _ = cap_handle.join();
    let _ = asr_handle.join();

    result
}
