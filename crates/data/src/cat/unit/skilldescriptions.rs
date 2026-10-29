use std::fmt;

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of skill description text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillDescriptionsError {
    /// The supplied bytes yielded no lines at all.
    EmptyFile,
}

impl fmt::Display for SkillDescriptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid skill descriptions."),
        }
    }
}

impl std::error::Error for SkillDescriptionsError {}

/// Localized skill descriptions, indexed by skill identifier.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillDescriptions {
    /// The sequence of cleansed description strings, indexed by skill ID.
    pub texts: Vec<String>,
}

impl SkillDescriptions {
    /// Parses the skill description table into a positional list of strings.
    ///
    /// Each line contributes one entry, with the leading identifier column
    /// stripped and embedded `<br>` markers converted to newlines. Blank lines
    /// contribute an empty entry rather than being skipped, so an entry's
    /// position in the list remains its skill identifier.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the skill description file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the structured `SkillDescriptions` on success, or a
    /// `SkillDescriptionsError` if the file contained no lines.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, SkillDescriptionsError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<SkillDescriptions, SkillDescriptionsError> {
    let file_content = file::scrub(bytes);
    let separator = file::resolve(separator, &file_content);

    let mut texts = Vec::new();

    for line in file_content.lines() {
        if line.trim().is_empty() {
            texts.push(String::new());
            continue;
        }

        let raw_text = match line.split_once(separator) {
            Some((_id, text_part)) => text_part,
            None => line,
        };

        texts.push(raw_text.replace("<br>", "\n").trim().to_string());
    }

    if texts.is_empty() {
        return Err(SkillDescriptionsError::EmptyFile);
    }

    Ok(SkillDescriptions { texts })
}