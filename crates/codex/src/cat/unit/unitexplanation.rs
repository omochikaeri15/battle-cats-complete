use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitExplanationError {
    EmptyFile,
}

impl fmt::Display for UnitExplanationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid explanation text."),
        }
    }
}

impl std::error::Error for UnitExplanationError {}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitExplanation {
    pub names: [Option<String>; 4],
    pub descriptions: [Option<Vec<String>>; 4],
}

impl UnitExplanation {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, UnitExplanationError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<UnitExplanation, UnitExplanationError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut names: [Option<String>; 4] = [const { None }; 4];
    let mut descriptions: [Option<Vec<String>>; 4] = [const { None }; 4];
    let mut has_content = false;

    for (line_index, file_line) in file_content.lines().enumerate().take(4) {
        let parts: Vec<&str> = file_line.split(separator_char).collect();
        
        if let Some(name_part) = parts.first() {
            let sanitized_name: String = name_part.trim().chars()
                .filter(|c| !is_problematic_char(*c))
                .collect();

            if !sanitized_name.is_empty() && !looks_like_garbage_id(&sanitized_name) {
                names[line_index] = Some(sanitized_name);
                has_content = true;
            }
        }

        let desc_lines: Vec<String> = parts.iter()
            .skip(1)
            .take(3)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !desc_lines.is_empty() {
            has_content = true;
            descriptions[line_index] = Some(desc_lines);
        }
    }

    if !has_content {
        return Err(UnitExplanationError::EmptyFile);
    }

    for form_index in 1..4 {
        let is_shifted = names[form_index].is_some()
            && descriptions[form_index - 1].as_ref().and_then(|prior| prior.first()) == names[form_index].as_ref();

        if is_shifted {
            names[form_index] = None;
            descriptions[form_index] = None;
        }
    }

    for form_index in 1..4 {
        if names[form_index].is_some()
            && names[form_index] == names[form_index - 1]
            && descriptions[form_index] == descriptions[form_index - 1]
        {
            names[form_index] = None;
            descriptions[form_index] = None;
        }
    }

    Ok(UnitExplanation { names, descriptions })
}

fn is_problematic_char(character: char) -> bool {
    let codepoint = character as u32;
    if (0xE0100..=0xE01EF).contains(&codepoint) { return true; }
    if (0xFE00..=0xFE0F).contains(&codepoint) { return true; }
    false
}

fn looks_like_garbage_id(text: &str) -> bool {
    text.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_missing_its_name_cell_drops_out() {
        let Ok(explanation) = UnitExplanation::parse("A|d1|d2|d3||\nB|e1|e2|e3||\ne1|e2|e3||", None) else {
            panic!("the explanation file parsed to no text");
        };

        assert_eq!(explanation.names[1].as_deref(), Some("B"));
        assert_eq!(explanation.names[2], None);
        assert_eq!(explanation.descriptions[2], None);
    }

    #[test]
    fn a_short_but_named_row_survives() {
        let Ok(explanation) = UnitExplanation::parse("Sardine|d1|d2|d3|\nSardine Cat|e1|e2|e3|", None) else {
            panic!("the explanation file parsed to no text");
        };

        assert_eq!(explanation.names[0].as_deref(), Some("Sardine"));
        assert_eq!(explanation.names[1].as_deref(), Some("Sardine Cat"));
        assert_eq!(explanation.descriptions[1], Some(vec!["e1".to_string(), "e2".to_string(), "e3".to_string()]));
    }

    #[test]
    fn a_verbatim_repeat_of_the_prior_form_is_deduplicated() {
        let Ok(explanation) = UnitExplanation::parse("A|d1|d2|d3||\nA|d1|d2|d3||", None) else {
            panic!("the explanation file parsed to no text");
        };

        assert_eq!(explanation.names[1], None);
        assert_eq!(explanation.descriptions[1], None);
    }

    #[test]
    fn a_normal_three_form_file_is_untouched() {
        let Ok(explanation) = UnitExplanation::parse("A|d1|d2|d3||\nB|e1|e2|e3||\nC|f1|f2|f3||", None) else {
            panic!("the explanation file parsed to no text");
        };

        assert_eq!(explanation.names[2].as_deref(), Some("C"));
        assert_eq!(explanation.descriptions[2], Some(vec!["f1".to_string(), "f2".to_string(), "f3".to_string()]));
    }
}
