#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Recognition,
    Segments,
    Logs,
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Recognition => " Recognition ",
            Screen::Segments => " Segments ",
            Screen::Logs => " Logs ",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Screen::Recognition => Screen::Segments,
            Screen::Segments => Screen::Logs,
            Screen::Logs => Screen::Recognition,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Screen::Recognition => Screen::Logs,
            Screen::Segments => Screen::Recognition,
            Screen::Logs => Screen::Segments,
        }
    }
}
