use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub struct VuMeter {
    rms: f32,
}

impl VuMeter {
    pub fn new(rms: f32) -> Self {
        Self { rms }
    }
}

fn rms_to_db(rms: f32) -> f32 {
    if rms <= 1e-10 {
        -60.0
    } else {
        (20.0 * rms.log10()).clamp(-60.0, 0.0)
    }
}

fn db_color(db: f32) -> Color {
    if db > -10.0 {
        Color::Red
    } else if db > -25.0 {
        Color::Yellow
    } else if db > -40.0 {
        Color::Green
    } else {
        Color::DarkGray
    }
}

impl Widget for VuMeter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 14 || area.height == 0 {
            return;
        }

        let db = rms_to_db(self.rms);
        let color = db_color(db);

        let total_bars = area.width.saturating_sub(10) as usize;
        let percent = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let filled = (percent * total_bars as f32) as usize;
        let empty = total_bars.saturating_sub(filled);

        let bar: String = format!("{}{}", "━".repeat(filled), "─".repeat(empty),);

        let db_str = format!("{:>4.0}dB", db);

        Paragraph::new(Line::from(vec![
            Span::styled("VOL", Style::new().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(bar, Style::new().fg(color)),
            Span::raw(" "),
            Span::styled(db_str, Style::new().fg(color)),
        ]))
        .render(area, buf);
    }
}
