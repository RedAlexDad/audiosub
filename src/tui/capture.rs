use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

use crate::tui::app::TuiApp;
use crate::tui::input;
use crate::tui::screen::Screen;
use crate::tui::view;

pub fn run_with_capture(
    app: &mut TuiApp,
    capture: &mut dyn crate::audio::AudioCapture,
    engine: &mut dyn crate::asr::AsrEngine,
    output: &mut crate::subtitle::SubtitleOutput,
    buffer: &mut crate::subtitle::SubtitleBuffer,
    chunk_size: usize,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = capture_loop(app, &mut terminal, capture, engine, output, buffer, chunk_size);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn capture_loop(
    app: &mut TuiApp,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    capture: &mut dyn crate::audio::AudioCapture,
    engine: &mut dyn crate::asr::AsrEngine,
    output: &mut crate::subtitle::SubtitleOutput,
    buffer: &mut crate::subtitle::SubtitleBuffer,
    chunk_size: usize,
) -> Result<()> {
    while app.running {
        terminal.draw(|f| view::render(app, f))?;

        if app.reset_requested {
            engine.reset()?;
            let _ = buffer.drain();
            app.reset_requested = false;
        }

        input::handle_input(app)?;

        if let Some(chunk) = capture.read(chunk_size)? {
            app.update_audio(chunk.data.len());
            app.update_audio_levels(&chunk.data);

            engine.feed_audio(&chunk.data)?;

            let partial = engine.partial_text()?;
            app.set_partial(&partial);

            let stream_pos_ms = (app.total_samples as u64 * 1000) / app.engine_rate as u64;

            let segments = engine.drain_segments()?;
            if !segments.is_empty() {
                app.add_segments(segments.clone());
                for seg in &segments {
                    buffer.push(seg.clone());
                }
            }

            for ready in buffer.flush(stream_pos_ms) {
                output.append(&ready)?;
            }
        }

        if app.screen == Screen::Logs {
            app.log_needs_refresh = true;
        }
    }

    let final_segments = engine.finalize()?;
    if !final_segments.is_empty() {
        app.add_segments(final_segments.clone());
        for seg in &final_segments {
            buffer.push(seg.clone());
        }
    }

    for ready in buffer.drain() {
        output.append(&ready)?;
    }

    output.close()?;

    terminal.draw(|f| view::render(app, f))?;
    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}
