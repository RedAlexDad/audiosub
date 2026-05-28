use crate::asr::Segment;
use anyhow::Result;

pub struct VttWriter;

impl Default for VttWriter {
    fn default() -> Self {
        Self
    }
}

impl VttWriter {
    pub fn new() -> Self {
        Self
    }
}

impl super::SubtitleWriter for VttWriter {
    fn write_header(&mut self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer, "WEBVTT\n")?;
        Ok(())
    }

    fn write_segment(&mut self, writer: &mut dyn std::io::Write, segment: &Segment, _index: usize) -> Result<()> {
        let start = ms_to_vtt(segment.start_ms);
        let end = ms_to_vtt(segment.end_ms);
        writeln!(writer, "{} --> {}", start, end)?;
        writeln!(writer, "{}\n", segment.text)?;
        Ok(())
    }

    fn write_footer(&mut self, _writer: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }
}

fn ms_to_vtt(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, millis)
}
