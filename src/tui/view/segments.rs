use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;

pub fn render_history(app: &TuiApp, frame: &mut Frame, area: Rect) {
    if area.height < 2 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" History ")
        .style(Style::new().fg(Color::DarkGray));

    let inner_height = area.height.saturating_sub(2) as usize;

    let items: Vec<Line> = app
        .partial_history
        .iter()
        .rev()
        .take(inner_height)
        .map(|s| Line::from(Span::styled(s, Style::new().fg(Color::Gray))))
        .collect();

    if items.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "(no partial results yet)",
                Style::new().fg(Color::DarkGray),
            )))
            .block(block),
            area,
        );
    } else {
        frame.render_widget(Paragraph::new(items).block(block).wrap(Wrap { trim: true }), area);
    }
}
