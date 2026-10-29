use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitEvolveError {
    EmptyFile,
}

impl fmt::Display for UnitEvolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid evolution text."),
        }
    }
}

impl std::error::Error for UnitEvolveError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitEvolve {
    pub texts: [Option<Vec<String>>; 4],
}

impl UnitEvolve {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<HashMap<u32, Self>, UnitEvolveError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<HashMap<u32, UnitEvolve>, UnitEvolveError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut map = HashMap::new();

    for (line_index, line_content) in file_content.lines().enumerate() {
        if line_content.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line_content.split(delimiter).collect();
        let cat_id = line_index as u32;

        let mut texts: [Option<Vec<String>>; 4] = [const { None }; 4];
        let mut has_content = false;

        let get_text = |index: usize| -> String {
            let raw_string = parts.get(index).map(|s| s.trim()).unwrap_or("");
            if raw_string == "@" || raw_string == "＠" {
                return String::new();
            }
            raw_string.replace("<br>", "\n")
        };

        let mut true_form: Vec<String> = [get_text(0), get_text(1), get_text(2)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        if !true_form.is_empty() {
            true_form.dedup();
            texts[2] = Some(true_form);
            has_content = true;
        }

        let mut ultra_form: Vec<String> = [get_text(4), get_text(5), get_text(6)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        if !ultra_form.is_empty() {
            ultra_form.dedup();
            texts[3] = Some(ultra_form);
            has_content = true;
        }

        for form_index in 1..4 {
            if texts[form_index].is_some() && texts[form_index] == texts[form_index - 1] {
                texts[form_index] = None;
            }
        }

        if has_content {
            map.insert(cat_id, UnitEvolve { texts });
        }
    }

    if map.is_empty() {
        return Err(UnitEvolveError::EmptyFile);
    }

    Ok(map)
}