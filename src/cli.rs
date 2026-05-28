use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "audiosub", version, about = "Real-time automatic subtitles for Linux")]
pub struct Cli {
    #[arg(long, short, env = "AUDIOSUB_CONFIG", help = "Path to config file")]
    pub config: Option<PathBuf>,

    #[arg(long, env = "AUDIOSUB_ENGINE", help = "ASR engine: vosk | whisper")]
    pub engine: Option<String>,

    #[arg(long, short, env = "AUDIOSUB_MODEL", help = "Model path")]
    pub model: Option<PathBuf>,

    #[arg(long, short, env = "AUDIOSUB_OUTPUT", help = "Output subtitle file")]
    pub output: Option<PathBuf>,

    #[arg(long, env = "AUDIOSUB_LANG", help = "Language code")]
    pub lang: Option<String>,

    #[arg(long, env = "AUDIOSUB_DEVICE", help = "Audio device (PulseAudio monitor source)")]
    pub device: Option<String>,

    #[arg(long, env = "AUDIOSUB_FORMAT", help = "Subtitle format: srt | vtt")]
    pub format: Option<String>,

    #[arg(long, short = 'T', help = "Enable TUI mode")]
    pub tui: bool,

    #[arg(long, help = "List available PulseAudio monitor sources and exit")]
    pub list_devices: bool,

    #[arg(
        long,
        short = 'd',
        env = "AUDIOSUB_DURATION",
        help = "Recording duration in seconds (default: unlimited)"
    )]
    pub duration: Option<u64>,

    #[arg(long, short = 'v', env = "AUDIOSUB_VERBOSE", action = clap::ArgAction::Count, help = "Verbosity level")]
    pub verbose: u8,
}
