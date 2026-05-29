use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "audiosub", version, about = "Real-time automatic subtitles for Linux")]
pub struct Cli {
    #[arg(long, short, env = "AUDIOSUB_CONFIG", help = "Path to config file")]
    pub config: Option<PathBuf>,

    #[arg(long, short, help = "Model path")]
    pub model: Option<PathBuf>,

    #[arg(long, short, help = "Output subtitle file")]
    pub output: Option<PathBuf>,

    #[arg(long, help = "Language code")]
    pub lang: Option<String>,

    #[arg(long, help = "Audio device (PulseAudio monitor source)")]
    pub device: Option<String>,

    #[arg(long, help = "Subtitle format: srt | vtt")]
    pub format: Option<String>,

    #[arg(long, help = "Max segment duration in milliseconds (default: 10000)")]
    pub max_duration: Option<u64>,

    #[arg(long, help = "Disable TUI (use plain CLI mode)")]
    pub no_tui: bool,

    #[arg(long, help = "List available PulseAudio monitor sources and exit")]
    pub list_devices: bool,

    #[arg(long, short = 'd', help = "Recording duration in seconds (default: unlimited)")]
    pub duration: Option<u64>,

    #[arg(long, short = 'v', action = clap::ArgAction::Count, help = "Verbosity level")]
    pub verbose: u8,
}
