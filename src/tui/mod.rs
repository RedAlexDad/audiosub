use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io;
use std::time::Duration;

use crate::asr::Segment;
use crate::subtitle;

fn ms_to_srt(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}

pub struct TuiApp {
    partial: String,
    segments: Vec<Segment>,
    segment_count: usize,
    total_samples: usize,
    engine_rate: u32,
    max_duration_ms: u64,
    elapsed: Duration,
    running: bool,
    scroll_offset: usize,
    auto_scroll: bool,
    paused: bool,
    message: Option<(String, u8)>,
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
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn export_srt(&mut self) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let path = Path::new("saved").join(format!("audiosub_{}.srt", now));
        fs::create_dir_all("saved")?;
        let mut file = File::create(&path)?;
        for (i, seg) in self.segments.iter().enumerate() {
            let start = ms_to_srt(seg.start_ms);
            let end = ms_to_srt(seg.end_ms);
            writeln!(file, "{}\n{} --> {}\n{}\n", i + 1, start, end, seg.text)?;
        }
        self.message = Some((format!("Saved: {}", path.display()), 15));
        Ok(())
    }

    fn export_txt(&mut self) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let path = Path::new("saved").join(format!("audiosub_{}.txt", now));
        fs::create_dir_all("saved")?;
        let mut file = File::create(&path)?;
        for seg in &self.segments {
            writeln!(file, "{}", seg.text)?;
        }
        self.message = Some((format!("Saved: {}", path.display()), 15));
        Ok(())
    }

    pub fn update_audio(&mut self, samples: usize) {
        if self.paused {
            return;
        }
        self.total_samples += samples;
        self.elapsed = Duration::from_secs_f64(self.total_samples as f64 / self.engine_rate as f64);
    }

    pub fn set_partial(&mut self, text: &str) {
        if self.paused {
            return;
        }
        if !text.is_empty() {
            self.partial = text.to_string();
        }
    }

    pub fn add_segments(&mut self, segments: Vec<Segment>) {
        if self.paused {
            return;
        }
        for seg in segments {
            for split in subtitle::split_segment(seg, self.max_duration_ms) {
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

    pub fn render(&mut self, frame: &mut Frame) {
        if let Some((_, ref mut ticks)) = self.message
            && *ticks > 0
        {
            *ticks -= 1;
            if *ticks == 0 {
                self.message = None;
            }
        }

        let area = frame.area();
        let [top, middle, bottom] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(18)]).areas(area);

        self.render_top(frame, top);
        self.render_middle(frame, middle);
        self.render_bottom(frame, bottom);
    }

    fn render_top(&self, frame: &mut Frame, area: Rect) {
        let status = if self.paused {
            Span::styled("⏸ PAUSED", Style::new().fg(Color::Yellow))
        } else if self.running {
            Span::styled("● RUNNING", Style::new().fg(Color::Green))
        } else {
            Span::styled("■ STOPPED", Style::new().fg(Color::Red))
        };

        let elapsed = format!("{:02}:{:02}", self.elapsed.as_secs() / 60, self.elapsed.as_secs() % 60);

        let title = Line::from(vec![
            Span::styled(" audiosub ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" │ "),
            status,
            Span::raw(" │ "),
            Span::raw(format!("{} segments", self.segment_count)),
            Span::raw(" │ "),
            Span::raw(elapsed),
            Span::raw(" │ "),
            Span::styled(" q/p/s/S/↑↓ ", Style::new().fg(Color::DarkGray)),
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::new().fg(Color::White));

        frame.render_widget(block, area);
    }

    fn render_middle(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Recognition ")
            .style(Style::new().fg(Color::Yellow));

        let text = if let Some((ref msg, _)) = self.message {
            msg.as_str()
        } else if self.partial.is_empty() {
            "Waiting for speech..."
        } else {
            &self.partial
        };

        let style = if self.message.is_some() {
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::White)
        };

        let paragraph = Paragraph::new(text).block(block).style(style).wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_bottom(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Segments ({}/{}) ",
                self.segments.len().saturating_sub(self.scroll_offset),
                self.segments.len()
            ))
            .style(Style::new().fg(Color::Blue));

        let inner_height = area.height.saturating_sub(2) as usize;
        let offset = self.scroll_offset.min(self.segments.len().saturating_sub(1));

        let items: Vec<Line> = self
            .segments
            .iter()
            .rev()
            .skip(offset)
            .take(inner_height)
            .map(|seg| {
                let ts = format!(
                    "{:02}:{:02} --> {:02}:{:02}",
                    seg.start_ms / 60000,
                    seg.start_ms % 60000 / 1000,
                    seg.end_ms / 60000,
                    seg.end_ms % 60000 / 1000,
                );
                Line::from(vec![
                    Span::styled(ts, Style::new().fg(Color::DarkGray)),
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

    pub fn handle_input(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key) = event::read()?
        {
            let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.stop();
                    return Ok(false);
                }
                KeyCode::Char('d') if is_ctrl => {
                    self.stop();
                    return Ok(false);
                }
                KeyCode::Char('p') => {
                    self.paused = !self.paused;
                    if self.paused {
                        self.auto_scroll = false;
                    } else {
                        self.auto_scroll = true;
                        self.scroll_offset = 0;
                    }
                }
                KeyCode::Char('S') => {
                    let _ = self.export_txt();
                }
                KeyCode::Char('s') => {
                    let _ = self.export_srt();
                }
                KeyCode::Up => {
                    if !self.segments.is_empty() {
                        self.auto_scroll = false;
                        self.scroll_offset = (self.scroll_offset + 1).min(self.segments.len().saturating_sub(1));
                    }
                }
                KeyCode::Down => {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                    if self.scroll_offset == 0 {
                        self.auto_scroll = true;
                    }
                }
                KeyCode::PageUp => {
                    if !self.segments.is_empty() {
                        self.auto_scroll = false;
                        self.scroll_offset = (self.scroll_offset + 10).min(self.segments.len().saturating_sub(1));
                    }
                }
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    if self.scroll_offset == 0 {
                        self.auto_scroll = true;
                    }
                }
                KeyCode::Home => {
                    if !self.segments.is_empty() {
                        self.auto_scroll = false;
                        self.scroll_offset = self.segments.len() - 1;
                    }
                }
                KeyCode::End => {
                    self.scroll_offset = 0;
                    self.auto_scroll = true;
                }
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn run_with_capture(
        &mut self,
        capture: &mut dyn crate::audio::AudioCapture,
        engine: &mut dyn crate::asr::AsrEngine,
        output: &mut crate::subtitle::SubtitleOutput,
        buffer: &mut crate::subtitle::SubtitleBuffer,
        chunk_size: usize,
    ) -> Result<()> {
        use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let result = self.capture_loop(&mut terminal, capture, engine, output, buffer, chunk_size);

        disable_raw_mode()?;
        crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn capture_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        capture: &mut dyn crate::audio::AudioCapture,
        engine: &mut dyn crate::asr::AsrEngine,
        output: &mut crate::subtitle::SubtitleOutput,
        buffer: &mut crate::subtitle::SubtitleBuffer,
        chunk_size: usize,
    ) -> Result<()> {
        while self.running {
            terminal.draw(|f| self.render(f))?;

            self.handle_input()?;

            if let Some(chunk) = capture.read(chunk_size)? {
                self.update_audio(chunk.data.len());

                engine.feed_audio(&chunk.data)?;

                let partial = engine.partial_text()?;
                self.set_partial(&partial);

                let stream_pos_ms = (self.total_samples as u64 * 1000) / self.engine_rate as u64;

                let segments = engine.drain_segments()?;
                if !segments.is_empty() {
                    self.add_segments(segments.clone());
                    for seg in &segments {
                        buffer.push(seg.clone());
                    }
                }

                for ready in buffer.flush(stream_pos_ms) {
                    output.append(&ready)?;
                }
            }
        }

        let final_segments = engine.finalize()?;
        if !final_segments.is_empty() {
            self.add_segments(final_segments.clone());
            for seg in &final_segments {
                buffer.push(seg.clone());
            }
        }

        for ready in buffer.drain() {
            output.append(&ready)?;
        }

        output.close()?;

        terminal.draw(|f| self.render(f))?;
        std::thread::sleep(Duration::from_secs(1));

        Ok(())
    }
}
