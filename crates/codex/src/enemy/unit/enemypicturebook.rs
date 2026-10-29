use std::error;
use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnemyPictureBook {
    pub description: Option<Vec<String>>,
}

impl EnemyPictureBook {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, EnemyPictureBookError> {
        parse_inner(bytes.as_ref(), separator)
    }

    pub fn parse_row<B: AsRef<[u8]>>(bytes: B, id: usize, separator: Option<Separator>) -> Option<Self> {
        parse_row_inner(bytes.as_ref(), id, separator)
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

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<EnemyPictureBook>, EnemyPictureBookError> {
    let content = file::scrub(bytes);
    let separator = file::resolve(separator, &content);

    let descriptions: Vec<EnemyPictureBook> = content
        .lines()
        .map(|line| parse_line_data(line, separator))
        .collect();

    if descriptions.is_empty() {
        return Err(EnemyPictureBookError::EmptyData);
    }

    Ok(descriptions)
}

fn parse_row_inner(bytes: &[u8], id: usize, separator: Option<Separator>) -> Option<EnemyPictureBook> {
    let content = file::scrub(bytes);
    let separator = file::resolve(separator, &content);

    content.lines().nth(id).map(|line| parse_line_data(line, separator))
}