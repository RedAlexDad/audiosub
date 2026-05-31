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

    let ph = app.partial_height.min(area.height.saturating_sub(3));
    let [partial_area, segments_area] = Layout::vertical([Constraint::Length(ph), Constraint::Fill(1)]).areas(area);

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

    let segments_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Segments ({}) ", app.segments.len()))
        .style(Style::new().fg(Color::Blue));

    let inner_height = segments_area.height.saturating_sub(2) as usize;
    let offset = app.scroll_offset.min(app.segments.len().saturating_sub(1));

    let items: Vec<Line> = app
        .segments
        .iter()
        .rev()
        .skip(offset)
        .take(inner_height)
        .enumerate()
        .map(|(idx, seg)| {
            let line_ts = format!("{}:{:02}", seg.start_ms / 60000, (seg.start_ms % 60000) / 1000);
            Line::from(vec![
                Span::styled(format!("{:>3} ", idx + offset + 1), Style::new().fg(Color::DarkGray)),
                Span::styled(line_ts, Style::new().fg(Color::Blue)),
                Span::raw("  "),
                Span::styled(&seg.text, Style::new().fg(Color::White)),
            ])
        })
        .collect();

    frame.render_widget(
        Paragraph::new(items)
            .block(segments_block)
            .style(Style::new().fg(Color::White))
            .wrap(Wrap { trim: false }),
        segments_area,
    );
}
