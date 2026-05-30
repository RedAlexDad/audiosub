use std::fs;
use std::path::Path;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;

pub fn refresh_logs(app: &mut TuiApp) {
    let path = Path::new("/tmp/audiosub_stderr.log");
    if !path.exists() {
        app.log_lines = vec!["(log file not found)".to_string()];
        return;
    }
    match fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            let tail = if lines.len() > 500 {
                lines[lines.len() - 500..].to_vec()
            } else {
                lines
            };
            app.log_lines = tail;
        }
        Err(e) => {
            app.log_lines = vec![format!("(error reading log: {e}")];
        }
    }
}

pub fn render_logs(app: &TuiApp, frame: &mut Frame, area: Rect) {
    if area.height < 2 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" stderr Log ")
        .style(Style::new().fg(Color::Red));

    let inner_height = area.height.saturating_sub(2) as usize;
    let log_scroll = app.log_lines.len().saturating_sub(inner_height);
    let items: Vec<Line> = app
        .log_lines
        .iter()
        .skip(log_scroll)
        .take(inner_height)
        .map(|s| Line::from(Span::styled(s, Style::new().fg(Color::Gray))))
        .collect();

    let paragraph = Paragraph::new(items)
        .block(block)
        .style(Style::new().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
