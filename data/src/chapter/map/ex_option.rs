//! Links from ordinary maps to the EX maps they can divert into.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of EX map links.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExOptionError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for ExOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid EX option data."
            ),
        }
    }
}

impl std::error::Error for ExOptionError {}

/// The parsed contents of the EX map link table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExOption {
    /// The identifier of the EX map each ordinary map can divert into, keyed by the ordinary map.
    pub map_to_ex_map: HashMap<u32, u32>,
}

impl ExOption {
    /// Parses the EX map link table into a mapping between map identifiers.
    ///
    /// Trailing comment text introduced by a double slash is discarded before
    /// the columns are read.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the EX option file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `ExOption` on success, or an
    /// `ExOptionError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, ExOptionError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<ExOption, ExOptionError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut map_to_ex_map = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines() {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 2 {
            continue;
        }

        let Some(map_id_string) = parts.first() else { continue; };
        let Ok(map_id) = map_id_string.trim().parse::<u32>() else { continue; };

        let Some(ex_map_id_string) = parts.get(1) else { continue; };
        let Ok(ex_map_id) = ex_map_id_string.trim().parse::<u32>() else { continue; };

        map_to_ex_map.insert(map_id, ex_map_id);
    }

    if !has_content {
        return Err(ExOptionError::EmptyFile);
    }

    Ok(ExOption { map_to_ex_map })
}