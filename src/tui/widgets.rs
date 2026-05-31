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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_to_db_silence() {
        assert_eq!(rms_to_db(0.0), -60.0);
        assert_eq!(rms_to_db(1e-10), -60.0);
    }

    #[test]
    fn rms_to_db_full_scale() {
        assert!((rms_to_db(1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn rms_to_db_half_scale() {
        let db = rms_to_db(0.5);
        assert!((db - (-6.0206)).abs() < 0.01);
    }

    #[test]
    fn rms_to_db_clamps() {
        assert_eq!(rms_to_db(1.0), 0.0);
        assert_eq!(rms_to_db(2.0), 0.0);
    }

    #[test]
    fn db_color_loud_is_red() {
        assert_eq!(db_color(-5.0), Color::Red);
        assert_eq!(db_color(-9.9), Color::Red);
    }

    #[test]
    fn db_color_medium_is_yellow() {
        assert_eq!(db_color(-10.0), Color::Yellow);
        assert_eq!(db_color(-15.0), Color::Yellow);
        assert_eq!(db_color(-24.9), Color::Yellow);
    }

    #[test]
    fn db_color_quiet_is_green() {
        assert_eq!(db_color(-25.0), Color::Green);
        assert_eq!(db_color(-30.0), Color::Green);
        assert_eq!(db_color(-39.9), Color::Green);
    }

    #[test]
    fn db_color_silent_is_dark_gray() {
        assert_eq!(db_color(-40.0), Color::DarkGray);
        assert_eq!(db_color(-45.0), Color::DarkGray);
        assert_eq!(db_color(-60.0), Color::DarkGray);
    }
}
