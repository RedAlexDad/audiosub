use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "audiosub", version, about = "Real-time automatic subtitles for Linux")]
pub struct Cli {
    #[arg(long, short, help = "Path to config file")]
    pub config: Option<PathBuf>,

    #[arg(long, default_value = "vosk", help = "ASR engine: vosk | whisper")]
    pub engine: Option<String>,

    #[arg(long, short, help = "Model path")]
    pub model: Option<PathBuf>,

    #[arg(long, short, default_value = "output.srt", help = "Output subtitle file")]
    pub output: Option<PathBuf>,

    #[arg(long, default_value = "en-US", help = "Language code")]
    pub lang: Option<String>,

    #[arg(long, help = "Audio device (PulseAudio monitor source)")]
    pub device: Option<String>,

    #[arg(long, help = "Subtitle format: srt | vtt")]
    pub format: Option<String>,

    #[arg(long, short = 'T', help = "Enable TUI mode (ratatui)")]
    pub tui: bool,

    #[arg(long, short = 'v', action = clap::ArgAction::Count, help = "Verbosity level")]
    pub verbose: u8,
}
