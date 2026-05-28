use crate::asr::Segment;
use anyhow::Result;
use std::io::Write;

pub trait SubtitleWriter: Send {
    fn write_header(&mut self, writer: &mut dyn Write) -> Result<()>;
    fn write_segment(&mut self, writer: &mut dyn Write, segment: &Segment, index: usize) -> Result<()>;
    fn write_footer(&mut self, writer: &mut dyn Write) -> Result<()>;
}

pub mod buffer;
pub mod output;
pub mod srt;
pub mod vtt;

pub use buffer::SubtitleBuffer;
pub use output::SubtitleOutput;

pub fn create_writer(format: &str) -> Box<dyn SubtitleWriter> {
    match format {
        "vtt" => Box::new(vtt::VttWriter),
        _ => Box::new(srt::SrtWriter::new()),
    }
}
