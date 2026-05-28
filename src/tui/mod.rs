use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::prelude::CrosstermBackend;
use ratatui::Frame;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

use crate::asr::Segment;

pub struct TuiApp {
    partial: String,
    segments: Vec<Segment>,
    segment_count: usize,
    total_samples: usize,
    engine_rate: u32,
    elapsed: Duration,
    running: bool,
}

impl TuiApp {
    pub fn new(engine_rate: u32) -> Self {
        Self {
            partial: String::new(),
            segments: Vec::new(),
            segment_count: 0,
            total_samples: 0,
            engine_rate,
            elapsed: Duration::default(),
            running: true,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update_audio(&mut self, samples: usize) {
        self.total_samples += samples;
        self.elapsed =
            Duration::from_secs_f64(self.total_samples as f64 / self.engine_rate as f64);
    }

    pub fn set_partial(&mut self, text: &str) {
        if !text.is_empty() {
            self.partial = text.to_string();
        }
    }

    pub fn add_segments(&mut self, segments: Vec<Segment>) {
        for seg in segments {
            self.segment_count += 1;
            self.segments.push(seg);
        }
        if self.segments.len() > 100 {
            self.segments.drain(..self.segments.len() - 100);
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [top, middle, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(10),
        ])
        .areas(area);

        self.render_top(frame, top);
        self.render_middle(frame, middle);
        self.render_bottom(frame, bottom);
    }

    fn render_top(&self, frame: &mut Frame, area: Rect) {
        let status = if self.running {
            Span::styled("● RUNNING", Style::new().fg(Color::Green))
        } else {
            Span::styled("■ STOPPED", Style::new().fg(Color::Red))
        };

        let elapsed = format!(
            "{:02}:{:02}",
            self.elapsed.as_secs() / 60,
            self.elapsed.as_secs() % 60
        );

        let title = Line::from(vec![
            Span::styled(" audiosub ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" │ "),
            status,
            Span::raw(" │ "),
            Span::raw(format!("{} segments", self.segment_count)),
            Span::raw(" │ "),
            Span::raw(elapsed),
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::new().fg(Color::White));

        frame.render_widget(block, area);
    }

    fn render_middle(&self, frame: &mut Frame, area: Rect) {
        let text = if self.partial.is_empty() {
            "Waiting for speech..."
        } else {
            &self.partial
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Recognition ")
            .style(Style::new().fg(Color::Yellow));

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::new().fg(Color::White))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_bottom(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<Line> = self
            .segments
            .iter()
            .rev()
            .take(5)
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

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Segments ")
            .style(Style::new().fg(Color::Blue));

        let paragraph = Paragraph::new(items)
            .block(block)
            .style(Style::new().fg(Color::White))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    pub fn handle_input(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.stop();
                        return Ok(false);
                    }
                    _ => {}
                }
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
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

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

                let stream_pos_ms =
                    (self.total_samples as u64 * 1000) / self.engine_rate as u64;

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
