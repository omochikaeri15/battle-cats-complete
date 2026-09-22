use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Slot(u8),
    Item(u8),
    Worker,
    Cannon,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    Key(Key, bool),
    Resize(f32, f32),
    Phone(bool),
    Move(i32, i32),
    Press(i32, i32),
    Release,
    Pinch(i32),
    Settle,
}

impl Cue {
    pub fn before_input(self) -> bool {
        matches!(self, Self::Key(..) | Self::Resize(..) | Self::Phone(_))
    }
}

impl Key {
    fn code(self) -> String {
        match self {
            Self::Slot(slot) => format!("s{slot}"),
            Self::Item(item) => format!("i{item}"),
            Self::Worker => "w".to_owned(),
            Self::Cannon => "c".to_owned(),
            Self::ZoomIn => "zi".to_owned(),
            Self::ZoomOut => "zo".to_owned(),
            Self::PanLeft => "pl".to_owned(),
            Self::PanRight => "pr".to_owned(),
            Self::Pause => "p".to_owned(),
        }
    }

    fn parse(code: &str) -> Option<Self> {
        match code {
            "w" => Some(Self::Worker),
            "c" => Some(Self::Cannon),
            "zi" => Some(Self::ZoomIn),
            "zo" => Some(Self::ZoomOut),
            "pl" => Some(Self::PanLeft),
            "pr" => Some(Self::PanRight),
            "p" => Some(Self::Pause),
            _ => {
                let (kind, index) = code.split_at_checked(1)?;
                let index = index.parse().ok()?;

                match kind {
                    "s" => Some(Self::Slot(index)),
                    "i" => Some(Self::Item(index)),
                    _ => None,
                }
            }
        }
    }
}

fn pair(text: &str, separator: char) -> Option<(i32, i32)> {
    let (x, y) = text.split_once(separator)?;

    Some((x.parse().ok()?, y.parse().ok()?))
}

impl fmt::Display for Cue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key, true) => write!(f, "+{}", key.code()),
            Self::Key(key, false) => write!(f, "-{}", key.code()),
            Self::Resize(width, height) => write!(f, "S{width}x{height}"),
            Self::Phone(phone) => write!(f, "T{}", u8::from(*phone)),
            Self::Move(x, y) => write!(f, "m{x},{y}"),
            Self::Press(x, y) => write!(f, "d{x},{y}"),
            Self::Release => f.write_char('r'),
            Self::Pinch(step) => write!(f, "z{step}"),
            Self::Settle => f.write_char('g'),
        }
    }
}

impl std::str::FromStr for Cue {
    type Err = String;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let parsed = match token.split_at_checked(1) {
            Some(("+", code)) => Key::parse(code).map(|key| Self::Key(key, true)),
            Some(("-", code)) => Key::parse(code).map(|key| Self::Key(key, false)),
            Some(("S", size)) => size
                .split_once('x')
                .and_then(|(width, height)| Some(Self::Resize(width.parse().ok()?, height.parse().ok()?))),
            Some(("T", "1")) => Some(Self::Phone(true)),
            Some(("T", "0")) => Some(Self::Phone(false)),
            Some(("m", at)) => pair(at, ',').map(|(x, y)| Self::Move(x, y)),
            Some(("d", at)) => pair(at, ',').map(|(x, y)| Self::Press(x, y)),
            Some(("r", "")) => Some(Self::Release),
            Some(("z", step)) => step.parse().ok().map(Self::Pinch),
            Some(("g", "")) => Some(Self::Settle),
            _ => None,
        };

        parsed.ok_or_else(|| format!("unreadable input cue {token:?}"))
    }
}

pub fn format_frame(cues: &[Cue]) -> String {
    let mut line = String::new();

    for (index, cue) in cues.iter().enumerate() {
        if index > 0 {
            line.push(' ');
        }

        let _ = write!(line, "{cue}");
    }

    line
}

pub fn parse(text: &str) -> Result<Vec<Vec<Cue>>, String> {
    text.lines()
        .enumerate()
        .map(|(frame, line)| {
            line.split_ascii_whitespace()
                .map(str::parse)
                .collect::<Result<Vec<Cue>, String>>()
                .map_err(|reason| format!("frame {frame}: {reason}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cue_survives_a_round_trip() {
        let frame = vec![
            Cue::Key(Key::Slot(9), true),
            Cue::Key(Key::Item(3), false),
            Cue::Key(Key::ZoomOut, true),
            Cue::Resize(1280.5, 720.0),
            Cue::Phone(true),
            Cue::Move(-4, 300),
            Cue::Press(12, -1),
            Cue::Release,
            Cue::Pinch(-28),
            Cue::Settle,
        ];
        let text = format!("{}\n\n{}\n", format_frame(&frame), format_frame(&[Cue::Release]));

        assert_eq!(parse(&text), Ok(vec![frame, Vec::new(), vec![Cue::Release]]));
    }

    #[test]
    fn a_bad_token_names_its_frame() {
        assert!(parse("r\nq9").is_err_and(|reason| reason.starts_with("frame 1")));
    }
}
