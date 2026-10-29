use std::collections::HashMap;
use std::fmt;

use crate::common::file;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalizableError {
    EmptyFile,
}

impl fmt::Display for LocalizableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalizableError::EmptyFile => write!(f, "The provided file bytes contained no valid localization data."),
        }
    }
}

impl std::error::Error for LocalizableError {}

#[derive(Debug, Clone, Default)]
pub struct Localizable {
    map: HashMap<String, String>,
}

impl Localizable {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, LocalizableError> {
        parse_inner(bytes.as_ref())
    }

    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Localizable, LocalizableError> {
    let content = file::scrub(bytes);
    let estimated_entries = bytes.len() / 50;
    let mut map = HashMap::with_capacity(estimated_entries);

    for line in content.lines() {
        let clean_line = line.split("//").next().unwrap_or("").trim();

        if clean_line.is_empty() {
            continue;
        }

        let Some(tab_index) = clean_line.find('\t') else {
            continue;
        };

        let key = clean_line[..tab_index].trim().to_string();
        let value = clean_line[tab_index..].trim().to_string();

        map.insert(key, value);
    }

    if map.is_empty() {
        return Err(LocalizableError::EmptyFile);
    }

    Ok(Localizable { map })
}