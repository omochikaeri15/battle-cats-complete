use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkillDescriptionsError {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillDescriptions {
    pub texts: Vec<String>,
}

impl SkillDescriptions {
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