use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::TuiApp;
use crate::tui::screen::Screen;

pub fn render_bottom(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let (title, items) = match app.screen {
        Screen::Recognition => (" Keys ", vec!["Tab: next screen  │  q: quit  │  p: pause  │  r: reset"]),
        Screen::Segments => (
            " Keys ",
            vec!["Tab: next screen  │  s: SRT  │  S: TXT  │  ↑↓: scroll  │  Del: remove last  │  c: clear all"],
        ),
        Screen::Logs => (" Keys ", vec!["Tab: next screen  │  q: quit  │  r: refresh logs"]),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::new().fg(Color::DarkGray));

    let lines: Vec<Line> = items
        .iter()
        .map(|s| Line::from(Span::styled(*s, Style::new().fg(Color::DarkGray))))
        .collect();

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
