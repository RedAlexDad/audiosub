use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use chrono::Local;

use crate::asr::Segment;

fn ms_to_srt(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}

pub fn export_srt(segments: &[Segment]) -> Result<String> {
    let now = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let path = Path::new("saved").join(format!("audiosub_{}.srt", now));
    fs::create_dir_all("saved")?;
    let mut file = File::create(&path)?;
    for (i, seg) in segments.iter().enumerate() {
        let start = ms_to_srt(seg.start_ms);
        let end = ms_to_srt(seg.end_ms);
        writeln!(file, "{}\n{} --> {}\n{}\n", i + 1, start, end, seg.text)?;
    }
    Ok(format!("SRT saved: {}", path.display()))
}

pub fn export_txt(segments: &[Segment]) -> Result<String> {
    let now = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let path = Path::new("saved").join(format!("audiosub_{}.txt", now));
    fs::create_dir_all("saved")?;
    let mut file = File::create(&path)?;
    for seg in segments {
        writeln!(file, "{}", seg.text)?;
    }
    Ok(format!("TXT saved: {}", path.display()))
}
