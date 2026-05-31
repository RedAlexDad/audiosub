use std::collections::VecDeque;
use std::time::Duration;

use crate::asr::Segment;
use crate::tui::screen::Screen;

pub struct TuiApp {
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
    pub fn new(engine_rate: u32, max_duration_ms: u64) -> Self {
        Self {
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

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn reset_requested(&self) -> bool {
        self.reset_requested
    }

    pub fn update_audio(&mut self, samples: usize) {
        if self.paused {
            return;
        }
        self.total_samples += samples;
        self.elapsed = Duration::from_secs_f64(self.total_samples as f64 / self.engine_rate as f64);
    }

    pub fn update_audio_levels(&mut self, data: &[f32]) {
        self.audio_level = Some((compute_rms(data), compute_peak(data)));
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

fn compute_rms(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = data.iter().map(|&s| s * s).sum();
    (sum_sq / data.len() as f32).sqrt().min(1.0)
}

fn compute_peak(data: &[f32]) -> f32 {
    data.iter().map(|&s| s.abs()).fold(0.0_f32, f32::max).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_rms_empty() {
        println!("Описание: пустой массив → RMS = 0.0");
        assert_eq!(compute_rms(&[]), 0.0);
    }

    #[test]
    fn compute_rms_silence() {
        println!("Описание: все нули → RMS = 0.0");
        let data = [0.0_f32; 100];
        assert_eq!(compute_rms(&data), 0.0);
    }

    #[test]
    fn compute_rms_max_amplitude() {
        println!("Описание: все 1.0 → RMS = 1.0");
        let data = [1.0_f32; 100];
        let rms = compute_rms(&data);
        assert!((rms - 1.0).abs() < 1e-6);
    }

    #[test]
    fn compute_rms_half_amplitude() {
        println!("Описание: все 0.5 → RMS = 0.5");
        let data = [0.5_f32; 100];
        let rms = compute_rms(&data);
        assert!((rms - 0.5).abs() < 1e-6);
    }

    #[test]
    fn compute_peak_empty() {
        println!("Описание: пустой массив → peak = 0.0");
        assert_eq!(compute_peak(&[]), 0.0);
    }

    #[test]
    fn compute_peak_silence() {
        println!("Описание: все нули → peak = 0.0");
        assert_eq!(compute_peak(&[0.0; 50]), 0.0);
    }

    #[test]
    fn compute_peak_max() {
        println!("Описание: максимальный по модулю sample = 1.0 → peak = 1.0");
        assert_eq!(compute_peak(&[0.0, 0.8, -1.0, 0.5]), 1.0);
    }

    #[test]
    fn compute_peak_never_exceeds_one() {
        println!("Описание: значения >1.0 клиппятся к 1.0");
        let data = vec![2.0, -3.0, 1.5];
        assert_eq!(compute_peak(&data), 1.0);
    }

    #[test]
    fn tui_app_new_state() {
        println!("Описание: TuiApp::new() создаёт состояние running, не reset, экран Recognition, пустые поля");
        let app = TuiApp::new(16000, 10000);
        assert!(app.is_running());
        assert!(!app.reset_requested());
        assert_eq!(app.screen, Screen::Recognition);
        assert!(app.partial.is_empty());
        assert!(app.segments.is_empty());
    }

    #[test]
    fn tui_app_stop() {
        println!("Описание: stop() переводит is_running в false");
        let mut app = TuiApp::new(16000, 10000);
        app.stop();
        assert!(!app.is_running());
    }

    #[test]
    fn tui_app_update_audio() {
        println!("Описание: update_audio(16000) добавляет сэмплы и обновляет elapsed = 1 сек");
        let mut app = TuiApp::new(16000, 10000);
        app.update_audio(16000);
        assert_eq!(app.total_samples, 16000);
        assert_eq!(app.elapsed.as_secs(), 1);
    }

    #[test]
    fn tui_app_update_audio_respects_paused() {
        println!("Описание: при paused=true сэмплы не учитываются, elapsed не растёт");
        let mut app = TuiApp::new(16000, 10000);
        app.paused = true;
        app.update_audio(16000);
        assert_eq!(app.total_samples, 0);
    }

    #[test]
    fn tui_app_set_partial() {
        println!("Описание: set_partial() обновляет partial и добавляет запись в partial_history");
        let mut app = TuiApp::new(16000, 10000);
        app.set_partial("hello world");
        assert_eq!(app.partial, "hello world");
        assert_eq!(app.partial_history.len(), 1);
    }

    #[test]
    fn tui_app_set_partial_skips_duplicates() {
        println!("Описание: повторный set_partial() с тем же текстом не дублирует историю");
        let mut app = TuiApp::new(16000, 10000);
        app.set_partial("hello");
        app.set_partial("hello");
        assert_eq!(app.partial_history.len(), 1);
    }

    #[test]
    fn tui_app_set_partial_respects_paused() {
        println!("Описание: при paused=true partial не обновляется");
        let mut app = TuiApp::new(16000, 10000);
        app.paused = true;
        app.set_partial("hello");
        assert!(app.partial.is_empty());
    }

    #[test]
    fn tui_app_add_segments() {
        println!("Описание: add_segments() добавляет сегменты и увеличивает счётчик");
        let mut app = TuiApp::new(16000, 10000);
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
        let mut app = TuiApp::new(16000, 10000);
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
        let mut app = TuiApp::new(16000, 10000);
        app.set_partial("hello");
        app.add_segments(vec![Segment {
            start_ms: 0,
            end_ms: 1000,
            text: "x".into(),
        }]);
        app.update_audio(16000);
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
        let mut app = TuiApp::new(16000, 10000);
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
