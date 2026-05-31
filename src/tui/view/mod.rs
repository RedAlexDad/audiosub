pub mod bottom;
pub mod logs;
pub mod recognition;
pub mod segments;
pub mod top;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::tui::app::TuiApp;
use crate::tui::screen::Screen;

pub fn render(app: &mut TuiApp, frame: &mut Frame) {
    if let Some((_, ref mut ticks)) = app.message
        && *ticks > 0
    {
        *ticks -= 1;
        if *ticks == 0 {
            app.message = None;
        }
    }

    if app.screen == Screen::Logs && app.log_needs_refresh {
        logs::refresh_logs(app);
        app.log_needs_refresh = false;
    }

    let area = frame.area();
    let [top_area, mid_bot] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

    top::render_top(app, frame, top_area);

    let [mid, bot] = Layout::vertical([Constraint::Fill(1), Constraint::Length(5)]).areas(mid_bot);

    match app.screen {
        Screen::Recognition => recognition::render_recognition(app, frame, mid),
        Screen::Segments => segments::render_history(app, frame, mid),
        Screen::Logs => logs::render_logs(app, frame, mid),
    }
    bottom::render_bottom(app, frame, bot);
}
