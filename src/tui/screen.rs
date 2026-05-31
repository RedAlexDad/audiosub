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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_names() {
        assert_eq!(Screen::Recognition.name(), " Recognition ");
        assert_eq!(Screen::Segments.name(), " Segments ");
        assert_eq!(Screen::Logs.name(), " Logs ");
    }

    #[test]
    fn screen_next_wraps() {
        assert_eq!(Screen::Recognition.next(), Screen::Segments);
        assert_eq!(Screen::Segments.next(), Screen::Logs);
        assert_eq!(Screen::Logs.next(), Screen::Recognition);
    }

    #[test]
    fn screen_prev_wraps() {
        assert_eq!(Screen::Recognition.prev(), Screen::Logs);
        assert_eq!(Screen::Segments.prev(), Screen::Recognition);
        assert_eq!(Screen::Logs.prev(), Screen::Segments);
    }

    #[test]
    fn screen_next_then_prev_returns_to_original() {
        for s in [Screen::Recognition, Screen::Segments, Screen::Logs] {
            assert_eq!(s.next().prev(), s);
            assert_eq!(s.prev().next(), s);
        }
    }
}
