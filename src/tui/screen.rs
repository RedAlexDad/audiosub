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
        println!("Описание: каждый Screen возвращает ожидаемое имя с пробелами");
        assert_eq!(Screen::Recognition.name(), " Recognition ");
        assert_eq!(Screen::Segments.name(), " Segments ");
        assert_eq!(Screen::Logs.name(), " Logs ");
    }

    #[test]
    fn screen_next_wraps() {
        println!("Описание: next() циклически: Recognition → Segments → Logs → Recognition");
        assert_eq!(Screen::Recognition.next(), Screen::Segments);
        assert_eq!(Screen::Segments.next(), Screen::Logs);
        assert_eq!(Screen::Logs.next(), Screen::Recognition);
    }

    #[test]
    fn screen_prev_wraps() {
        println!("Описание: prev() циклически: Recognition → Logs → Segments → Recognition");
        assert_eq!(Screen::Recognition.prev(), Screen::Logs);
        assert_eq!(Screen::Segments.prev(), Screen::Recognition);
        assert_eq!(Screen::Logs.prev(), Screen::Segments);
    }

    #[test]
    fn screen_next_then_prev_returns_to_original() {
        println!("Описание: для всех экранов next().prev() и prev().next() возвращают исходный");
        for s in [Screen::Recognition, Screen::Segments, Screen::Logs] {
            assert_eq!(s.next().prev(), s);
            assert_eq!(s.prev().next(), s);
        }
    }
}
