use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders};

use crate::tui::app::TuiApp;
use crate::tui::widgets::VuMeter;

pub fn render_top(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let status = if app.paused {
        Span::styled("⏸ PAUSED", Style::new().fg(Color::Yellow))
    } else if app.running {
        Span::styled("● RUNNING", Style::new().fg(Color::Green))
    } else {
        Span::styled("■ STOPPED", Style::new().fg(Color::Red))
    };

    let elapsed = format!("{:02}:{:02}", app.elapsed.as_secs() / 60, app.elapsed.as_secs() % 60);

    let title = Line::from(vec![
        Span::styled(" audiosub ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        status,
        Span::raw(" │ "),
        Span::raw(format!("{} segs", app.segment_count)),
        Span::raw(" │ "),
        Span::raw(elapsed),
        Span::raw(" │ "),
        Span::styled(app.screen.name().trim(), Style::new().fg(Color::Magenta)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::new().fg(Color::White));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some((rms, _peak)) = app.audio_level
        && inner.height >= 1
    {
        let vu_area = Rect {
            x: inner.x + inner.width.saturating_sub(26).min(inner.width),
            y: inner.y,
            width: 26.min(inner.width),
            height: 1,
        };
        frame.render_widget(VuMeter::new(rms), vu_area);
    }
}
