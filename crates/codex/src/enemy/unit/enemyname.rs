use std::error;
use std::fmt;

use crate::common::file;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnemyNameError {
    EmptyData,
}

impl fmt::Display for EnemyNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "The provided enemy name file contained no valid entries."),
        }
    }
}

impl error::Error for EnemyNameError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnemyName {
    pub name: Option<String>,
}

impl EnemyName {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Self>, EnemyNameError> {
        parse_inner(bytes.as_ref())
    }

    pub fn parse_row<B: AsRef<[u8]>>(bytes: B, id: usize) -> Option<Self> {
        parse_row_inner(bytes.as_ref(), id)
    }
}

fn parse_line_data(line: &str) -> EnemyName {
    let raw_name = line.trim().to_string();
    let is_invalid = raw_name.is_empty() || raw_name == "ダミー";

    EnemyName {
        name: if is_invalid { None } else { Some(raw_name) },
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Vec<EnemyName>, EnemyNameError> {
    let content = file::scrub(bytes);

    let names: Vec<EnemyName> = content.lines().map(parse_line_data).collect();

    if names.is_empty() {
        return Err(EnemyNameError::EmptyData);
    }

    Ok(names)
}

fn parse_row_inner(bytes: &[u8], id: usize) -> Option<EnemyName> {
    let content = file::scrub(bytes);

    content.lines().nth(id).map(parse_line_data)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_carrying_a_comma_survives_whole() {
        let Ok(names) = EnemyName::parse("Chef Bun Bun\nWinged Pigge, le Terrible\n") else {
            panic!("the terminology file parsed to no entries");
        };

        assert_eq!(names[1].name.as_deref(), Some("Winged Pigge, le Terrible"));
        assert_eq!(EnemyName::parse_row("a\nWinged Pigge, le Terrible", 1), Some(names[1].clone()));
    }
}
