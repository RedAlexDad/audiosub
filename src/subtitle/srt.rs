use crate::asr::Segment;
use anyhow::Result;

pub struct SrtWriter {
    index: usize,
}

impl Default for SrtWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl SrtWriter {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl super::SubtitleWriter for SrtWriter {
    fn write_header(&mut self, _writer: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }

    fn write_segment(&mut self, writer: &mut dyn std::io::Write, segment: &Segment, _index: usize) -> Result<()> {
        self.index += 1;
        let start = ms_to_srt(segment.start_ms);
        let end = ms_to_srt(segment.end_ms);
        writeln!(writer, "{}", self.index)?;
        writeln!(writer, "{} --> {}", start, end)?;
        writeln!(writer, "{}\n", segment.text)?;
        Ok(())
    }

    fn write_footer(&mut self, _writer: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }
}

fn ms_to_srt(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_srt_zero() {
        assert_eq!(ms_to_srt(0), "00:00:00,000");
    }

    #[test]
    fn ms_to_srt_millis() {
        assert_eq!(ms_to_srt(1234), "00:00:01,234");
    }

    #[test]
    fn ms_to_srt_minutes() {
        assert_eq!(ms_to_srt(65_000), "00:01:05,000");
    }

    #[test]
    fn ms_to_srt_hours() {
        assert_eq!(ms_to_srt(3_721_500), "01:02:01,500");
    }

    #[test]
    fn ms_to_srt_large() {
        assert_eq!(ms_to_srt(90_000_000), "25:00:00,000");
    }
}
