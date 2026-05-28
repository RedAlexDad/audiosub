use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::{create_writer, SubtitleWriter};
use crate::asr::Segment;

pub struct SubtitleOutput {
    writer: Box<dyn SubtitleWriter>,
    file: BufWriter<File>,
    index: usize,
}

impl SubtitleOutput {
    pub fn create(path: &Path, format: &str) -> Result<Self> {
        let file = File::create(path)
            .with_context(|| format!("Failed to create subtitle file: {:?}", path))?;
        let mut file = BufWriter::new(file);

        let mut writer = create_writer(format);
        writer.write_header(&mut file)?;
        file.flush()?;

        Ok(Self { writer, file, index: 0 })
    }

    pub fn append(&mut self, segment: &Segment) -> Result<()> {
        self.index += 1;
        self.writer
            .write_segment(&mut self.file, segment, self.index)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn append_all(&mut self, segments: &[Segment]) -> Result<()> {
        for seg in segments {
            self.append(seg)?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.writer.write_footer(&mut self.file)?;
        self.file.flush()?;
        Ok(())
    }
}

impl Drop for SubtitleOutput {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
