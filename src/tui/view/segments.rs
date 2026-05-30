use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;

pub fn render_segments_list(app: &TuiApp, frame: &mut Frame, area: Rect) {
    if area.height < 2 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Segments ({}) ", app.segments.len()))
        .style(Style::new().fg(Color::Blue));

    let inner_height = area.height.saturating_sub(2) as usize;
    let offset = app.scroll_offset.min(app.segments.len().saturating_sub(1));

    let items: Vec<Line> = app
        .segments
        .iter()
        .rev()
        .skip(offset)
        .take(inner_height)
        .enumerate()
        .map(|(idx, seg)| {
            let line_ts = format!("{}:{:02}", seg.start_ms / 60000, (seg.start_ms % 60000) / 1000,);
            Line::from(vec![
                Span::styled(format!("{:>3} ", idx + offset + 1), Style::new().fg(Color::DarkGray)),
                Span::styled(line_ts, Style::new().fg(Color::Blue)),
                Span::raw("  "),
                Span::styled(&seg.text, Style::new().fg(Color::White)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(items)
        .block(block)
        .style(Style::new().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
