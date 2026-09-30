use std::error;
use std::fmt;

use crate::common::file;

/// Represents errors that can occur during the parsing of localized enemy names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnemyNameError {
    /// The supplied bytes yielded no lines at all.
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

/// An enemy's localized display name.
///
/// The file carries no delimiter: every line is one whole name, and several
/// localized names contain a comma. Developer placeholders such as "ダミー" are
/// rejected, so an unnamed or placeholder enemy evaluates to `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnemyName {
    /// The parsed display name. `None` if the name is a placeholder or empty.
    pub name: Option<String>,
}

impl EnemyName {
    /// Parses the enemy terminology table into one entry per declared enemy.
    ///
    /// Every line contributes an entry, including blank ones, so an entry's
    /// position in the returned vector is its internal enemy identifier. The
    /// line is taken whole, since the file declares no delimiter.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the terminology file.
    ///
    /// # Returns
    /// A `Result` containing the parsed names indexed by enemy identifier on
    /// success, or an `EnemyNameError` if the file contained no lines.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Self>, EnemyNameError> {
        parse_inner(bytes.as_ref())
    }

    /// Parses a single row of the enemy terminology table by enemy identifier.
    ///
    /// This avoids materializing the entire table when only one name is
    /// required.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the terminology file.
    /// * `id` - The internal enemy identifier, used as a zero-based line offset.
    ///
    /// # Returns
    /// An `Option` containing the parsed name, or `None` if the identifier lies
    /// beyond the end of the table.
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
