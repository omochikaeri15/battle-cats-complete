use std::fmt;
use std::error;
use crate::common::utils::csv;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnemyName {
    pub name: Option<String>,
}

impl EnemyName {
    pub fn parse_all<T: AsRef<[u8]>>(b: T) -> Result<Vec<Self>, EnemyNameError> {
        parse_all_inner(b.as_ref())
    }

    pub fn parse<T: AsRef<[u8]>>(b: T, id: usize) -> Result<Option<Self>, EnemyNameError> {
        parse_inner(b.as_ref(), id)
    }
}

fn parse_line_data(line: &str, separator: char) -> EnemyName {
    let raw_name = line.split(separator).next().unwrap_or("").trim().to_string();
    let is_invalid = raw_name.is_empty() || raw_name == "ダミー";

    EnemyName {
        name: if is_invalid { None } else { Some(raw_name) },
    }
}

fn parse_all_inner(bytes: &[u8]) -> Result<Vec<EnemyName>, EnemyNameError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);
    let mut names = Vec::new();

    for line in content.lines() {
        names.push(parse_line_data(line, separator));
    }

    if names.is_empty() {
        return Err(EnemyNameError::EmptyData);
    }

    Ok(names)
}

fn parse_inner(bytes: &[u8], id: usize) -> Result<Option<EnemyName>, EnemyNameError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);

    let Some(target_line) = content.lines().nth(id) else {
        return Ok(None);
    };

    Ok(Some(parse_line_data(target_line, separator)))
}