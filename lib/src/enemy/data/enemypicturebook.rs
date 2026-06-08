use std::fmt;
use std::error;
use crate::common::utils::csv;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyPictureBookError {
    EmptyData,
}

impl fmt::Display for EnemyPictureBookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "The provided enemy picture book file contained no valid entries."),
        }
    }
}

impl error::Error for EnemyPictureBookError {}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnemyPictureBook {
    pub description: Option<Vec<String>>,
}

impl EnemyPictureBook {
    pub fn parse_all<T: AsRef<[u8]>>(b: T) -> Result<Vec<Self>, EnemyPictureBookError> {
        parse_all_inner(b.as_ref())
    }

    pub fn parse<T: AsRef<[u8]>>(b: T, id: usize) -> Result<Option<Self>, EnemyPictureBookError> {
        parse_inner(b.as_ref(), id)
    }
}

fn parse_line_data(line: &str, separator: char) -> EnemyPictureBook {
    let cols: Vec<&str> = line.split(separator).collect();
    let mut desc_lines = Vec::new();

    for col in cols.into_iter().skip(1) {
        let text = col.trim();
        if text.is_empty() || text.starts_with("仮") { continue; }
        desc_lines.push(text.to_string());
    }

    EnemyPictureBook {
        description: if desc_lines.is_empty() { None } else { Some(desc_lines) },
    }
}

fn parse_all_inner(bytes: &[u8]) -> Result<Vec<EnemyPictureBook>, EnemyPictureBookError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);
    let mut descriptions = Vec::new();

    for line in content.lines() {
        descriptions.push(parse_line_data(line, separator));
    }

    if descriptions.is_empty() {
        return Err(EnemyPictureBookError::EmptyData);
    }

    Ok(descriptions)
}

fn parse_inner(bytes: &[u8], id: usize) -> Result<Option<EnemyPictureBook>, EnemyPictureBookError> {
    let content = csv::scrub(bytes);
    let separator = csv::detect_separator(&content);

    let Some(target_line) = content.lines().nth(id) else {
        return Ok(None);
    };

    Ok(Some(parse_line_data(target_line, separator)))
}