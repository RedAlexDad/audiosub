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
            if text != prev {
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
