use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;

pub fn render_recognition(app: &TuiApp, frame: &mut Frame, area: Rect) {
    if area.height < 2 {
        return;
    }

    let [partial_area, history_area] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

    let partial_block = Block::default()
        .borders(Borders::ALL)
        .title(" Current ")
        .style(Style::new().fg(Color::Yellow));

    let partial_text = if let Some((ref msg, _)) = app.message {
        msg.as_str()
    } else if app.partial.is_empty() {
        "Waiting for speech..."
    } else {
        &app.partial
    };

    let pstyle = if app.message.is_some() {
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::White)
    };

    frame.render_widget(
        Paragraph::new(partial_text)
            .block(partial_block)
            .style(pstyle)
            .wrap(Wrap { trim: true }),
        partial_area,
    );

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title(" History ")
        .style(Style::new().fg(Color::DarkGray));

    let history_lines: Vec<Line> = app
        .partial_history
        .iter()
        .rev()
        .take(area.height.saturating_sub(6) as usize)
        .map(|s| Line::from(Span::styled(s, Style::new().fg(Color::Gray))))
        .collect();

    if history_lines.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "(no partial results yet)",
                Style::new().fg(Color::DarkGray),
            )))
            .block(history_block),
            history_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new(history_lines)
                .block(history_block)
                .wrap(Wrap { trim: true }),
            history_area,
        );
    }
}
