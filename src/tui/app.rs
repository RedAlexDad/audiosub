use std::collections::VecDeque;
use std::time::Duration;

use crate::asr::Segment;
use crate::tui::screen::Screen;

pub struct TuiApp {
    pub engine_name: String,
    pub partial: String,
    pub segments: Vec<Segment>,
    pub segment_count: usize,
    pub total_samples: usize,
    pub engine_rate: u32,
    pub max_duration_ms: u64,
    pub elapsed: Duration,
    pub running: bool,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub paused: bool,
    pub message: Option<(String, u8)>,
    pub audio_level: Option<(f32, f32)>,
    pub screen: Screen,
    pub partial_history: VecDeque<String>,
    pub log_lines: Vec<String>,
    pub log_needs_refresh: bool,
    pub reset_requested: bool,
    pub partial_height: u16,
    pub drag_start: Option<(u16, u16)>,
}

impl TuiApp {
    pub fn new(engine_name: &str, engine_rate: u32, max_duration_ms: u64) -> Self {
        Self {
            engine_name: engine_name.to_string(),
            partial: String::new(),
            segments: Vec::new(),
            segment_count: 0,
            total_samples: 0,
            engine_rate,
            max_duration_ms,
            elapsed: Duration::default(),
            running: true,
            scroll_offset: 0,
            auto_scroll: true,
            paused: false,
            message: None,
            audio_level: None,
            screen: Screen::Recognition,
            partial_history: VecDeque::with_capacity(30),
            log_lines: Vec::new(),
            log_needs_refresh: true,
            reset_requested: false,
            partial_height: 3,
            drag_start: None,
        }
    }

    pub fn set_partial(&mut self, text: &str) {
        if self.paused {
            return;
        }
        if !text.is_empty() {
            let prev = self.partial_history.back().map(|s| s.as_str()).unwrap_or("");
            if text != prev && (prev.is_empty() || !(text.starts_with(prev) || prev.starts_with(text))) {
                self.partial_history.push_back(text.to_string());
                if self.partial_history.len() > 30 {
                    self.partial_history.pop_front();
                }
            }
            self.partial = text.to_string();
        }
    }

    pub fn add_segments(&mut self, segments: Vec<Segment>) {
        if self.paused {
            return;
        }
        for seg in segments {
            for split in crate::subtitle::split_segment(seg, self.max_duration_ms) {
                self.segment_count += 1;
                self.segments.push(split);
            }
        }
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn do_reset(&mut self) {
        self.segments.clear();
        self.segment_count = 0;
        self.total_samples = 0;
        self.elapsed = Duration::default();
        self.scroll_offset = 0;
        self.auto_scroll = true;
        self.paused = false;
        self.partial.clear();
        self.partial_history.clear();
        self.audio_level = None;
        self.message = Some(("Reset".to_string(), 8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_app_set_partial() {
        println!("Описание: set_partial() обновляет partial и добавляет запись в partial_history");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.set_partial("hello world");
        assert_eq!(app.partial, "hello world");
        assert_eq!(app.partial_history.len(), 1);
    }

    #[test]
    fn tui_app_set_partial_skips_duplicates() {
        println!("Описание: повторный set_partial() с тем же текстом не дублирует историю");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.set_partial("hello");
        app.set_partial("hello");
        assert_eq!(app.partial_history.len(), 1);
    }

    #[test]
    fn tui_app_set_partial_respects_paused() {
        println!("Описание: при paused=true partial не обновляется");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.paused = true;
        app.set_partial("hello");
        assert!(app.partial.is_empty());
    }

    #[test]
    fn tui_app_add_segments() {
        println!("Описание: add_segments() добавляет сегменты и увеличивает счётчик");
        let mut app = TuiApp::new("test", 16000, 10000);
        let segs = vec![
            Segment {
                start_ms: 0,
                end_ms: 1000,
                text: "one".into(),
            },
            Segment {
                start_ms: 1000,
                end_ms: 2000,
                text: "two".into(),
            },
        ];
        app.add_segments(segs);
        assert_eq!(app.segments.len(), 2);
        assert_eq!(app.segment_count, 2);
    }

    #[test]
    fn tui_app_add_segments_respects_paused() {
        println!("Описание: при paused=true сегменты не добавляются");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.paused = true;
        app.add_segments(vec![Segment {
            start_ms: 0,
            end_ms: 1000,
            text: "x".into(),
        }]);
        assert!(app.segments.is_empty());
    }

    #[test]
    fn tui_app_do_reset() {
        println!("Описание: do_reset() очищает сегменты, историю, partial, счётчики и audio_level");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.set_partial("hello");
        app.add_segments(vec![Segment {
            start_ms: 0,
            end_ms: 1000,
            text: "x".into(),
        }]);
        app.total_samples = 16000;
        app.elapsed = Duration::from_secs_f64(1.0);
        app.do_reset();

        assert!(app.segments.is_empty());
        assert_eq!(app.segment_count, 0);
        assert_eq!(app.total_samples, 0);
        assert!(app.partial.is_empty());
        assert!(app.partial_history.is_empty());
        assert!(app.audio_level.is_none());
    }

    #[test]
    fn tui_app_auto_scroll_triggers_on_add() {
        println!("Описание: при auto_scroll=true добавление сегмента сбрасывает scroll_offset в 0");
        let mut app = TuiApp::new("test", 16000, 10000);
        app.scroll_offset = 5;
        app.auto_scroll = true;
        app.add_segments(vec![Segment {
            start_ms: 0,
            end_ms: 1000,
            text: "x".into(),
        }]);
        assert_eq!(app.scroll_offset, 0);
    }
}
