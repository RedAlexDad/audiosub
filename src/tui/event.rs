use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

use crate::tui::app::TuiApp;
use crate::tui::screen::Screen;

pub fn handle_input(app: &mut TuiApp) -> Result<bool> {
    if !event::poll(Duration::from_millis(0))? {
        return Ok(true);
    }

    match event::read()? {
        Event::Key(key) => {
            let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.stop();
                    return Ok(false);
                }
                KeyCode::Char('d') if is_ctrl => {
                    app.stop();
                    return Ok(false);
                }
                KeyCode::Tab => {
                    app.screen = app.screen.next();
                }
                KeyCode::BackTab => {
                    app.screen = app.screen.prev();
                }
                KeyCode::Char('p') => {
                    app.paused = !app.paused;
                    if app.paused {
                        app.auto_scroll = false;
                    } else {
                        app.auto_scroll = true;
                        app.scroll_offset = 0;
                    }
                }
                KeyCode::Char('r') => {
                    app.do_reset();
                    app.reset_requested = true;
                }
                KeyCode::Char('S') => {
                    let _ = crate::tui::export::export_txt(&app.segments).map(|msg| app.message = Some((msg, 15)));
                }
                KeyCode::Char('s') => {
                    let _ = crate::tui::export::export_srt(&app.segments).map(|msg| app.message = Some((msg, 15)));
                }
                KeyCode::Char('c') if app.screen == Screen::Segments => {
                    app.do_reset();
                }
                KeyCode::Char('R') if app.screen == Screen::Logs => {
                    crate::tui::view::logs::refresh_logs(app);
                }
                KeyCode::Char('d') | KeyCode::Delete if app.screen == Screen::Segments => {
                    app.segments.pop();
                    app.segment_count = app.segment_count.saturating_sub(1);
                }
                KeyCode::Up => {
                    if !app.segments.is_empty() {
                        app.auto_scroll = false;
                        app.scroll_offset = app
                            .scroll_offset
                            .saturating_add(1)
                            .min(app.segments.len().saturating_sub(1));
                    }
                }
                KeyCode::Down => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(1);
                    if app.scroll_offset == 0 {
                        app.auto_scroll = true;
                    }
                }
                KeyCode::PageUp => {
                    if !app.segments.is_empty() {
                        app.auto_scroll = false;
                        app.scroll_offset = app
                            .scroll_offset
                            .saturating_add(10)
                            .min(app.segments.len().saturating_sub(1));
                    }
                }
                KeyCode::PageDown => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(10);
                    if app.scroll_offset == 0 {
                        app.auto_scroll = true;
                    }
                }
                KeyCode::Home => {
                    if !app.segments.is_empty() {
                        app.auto_scroll = false;
                        app.scroll_offset = app.segments.len() - 1;
                    }
                }
                KeyCode::End => {
                    app.scroll_offset = 0;
                    app.auto_scroll = true;
                }
                _ => {}
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(_) if app.screen == Screen::Recognition => {
                app.drag_start = Some((mouse.row, app.partial_height));
            }
            MouseEventKind::Drag(_) => {
                if let Some((start_y, base_height)) = app.drag_start {
                    let delta = mouse.row as i16 - start_y as i16;
                    let new_height = (base_height as i16 + delta).clamp(3, 80) as u16;
                    app.partial_height = new_height;
                }
            }
            MouseEventKind::Up(_) => {
                app.drag_start = None;
            }
            _ => {}
        },
        _ => {}
    }
    Ok(true)
}
