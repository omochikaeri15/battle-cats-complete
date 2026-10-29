use serde::{Deserialize, Serialize};

const JAPANESE: &str = "ja";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Separator {
    Pipe,
    Tab,
    Comma,
}

impl Separator {
    pub const ALL: [Self; 3] = [Self::Pipe, Self::Tab, Self::Comma];

    pub fn detect(text: &str) -> Option<Self> {
        if text.contains('|') {
            return Some(Self::Pipe);
        }

        text.chars().find_map(|current| match current {
            '\t' => Some(Self::Tab),
            ',' => Some(Self::Comma),
            _ => None,
        })
    }

    pub fn localized(name: &str) -> Self {
        let stem = name.rsplit_once('.').map_or(name, |(head, _)| head);
        let japanese = stem.rsplit_once('_').is_some_and(|(_, code)| code == JAPANESE);

        if japanese { Self::Comma } else { Self::Pipe }
    }

    pub const fn char(self) -> char {
        match self {
            Self::Pipe => '|',
            Self::Tab => '\t',
            Self::Comma => ',',
        }
    }

    pub fn split(self, line: &str) -> impl Iterator<Item = &str> {
        line.split(self.char())
    }
}

pub(crate) fn resolve(separator: Option<Separator>, text: &str) -> char {
    separator
        .or_else(|| Separator::detect(text))
        .unwrap_or(Separator::Comma)
        .char()
}

pub fn scrub(bytes: &[u8]) -> String {
    let raw_text = String::from_utf8_lossy(bytes);
    let mut clean_text = String::with_capacity(raw_text.len());
    let mut char_stream = raw_text.chars().peekable();

    while let Some(current) = char_stream.next() {
        if matches!(current, '\u{feff}' | '\0') {
            continue;
        }

        if current == '\r' {
            clean_text.push('\n');
            if let Some(&'\n') = char_stream.peek() {
                char_stream.next();
            }
            continue;
        }

        clean_text.push(current);
    }

    clean_text
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakHandling {
    Space,
    Newline,
    Delete,
}

pub fn strip_html_tags(input: &str, handling: BreakHandling) -> String {
    let mut stripped = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find('<') {
        stripped.push_str(&rest[..start]);
        rest = &rest[start..];

        let Some(end) = rest.find('>') else {
            break;
        };

        let tag = &rest[..=end];
        rest = &rest[end + 1..];

        if tag.len() < 3 || !tag[..3].eq_ignore_ascii_case("<br") {
            continue;
        }

        match handling {
            BreakHandling::Space => {
                if !stripped.ends_with(' ') && !rest.starts_with(' ') {
                    stripped.push(' ');
                }
            }
            BreakHandling::Newline => stripped.push('\n'),
            BreakHandling::Delete => continue,
        }
    }

    stripped.push_str(rest);
    stripped
}

pub fn lookup(
    data: &[u8],
    key: &str,
    search_col: usize,
    target_col: usize,
    separator: Option<Separator>,
) -> Option<String> {
    let content = scrub(data);
    let separator = resolve(separator, &content);

    for line in content.lines() {
        let Some(clean_line) = line.split("//").next() else {
            continue;
        };

        let trimmed_line = clean_line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        let Some(current_key) = trimmed_line.split(separator).nth(search_col) else {
            continue;
        };

        if current_key.trim() != key {
            continue;
        }

        let Some(target_value) = trimmed_line.split(separator).nth(target_col) else {
            continue;
        };

        return Some(target_value.trim().to_string());
    }

    None
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_anywhere_outranks_an_earlier_comma() {
        assert_eq!(Separator::detect("Soracte, mentor mental|text"), Some(Separator::Pipe));
        assert_eq!(Separator::detect("a\tb,c"), Some(Separator::Tab));
        assert_eq!(Separator::detect("a,b\tc"), Some(Separator::Comma));
        assert_eq!(Separator::detect("one whole name"), None);
    }

    #[test]
    fn a_stated_delimiter_overrides_the_content() {
        let shredded = "Soracte, mentor mental|Un mentor.";
        assert_eq!(Separator::Pipe.split(shredded).count(), 2);
        assert_eq!(Separator::detect(shredded).map(Separator::char), Some('|'));
        assert_eq!(resolve(Some(Separator::Comma), shredded), ',');
        assert_eq!(resolve(None, shredded), '|');
    }

    #[test]
    fn only_a_ja_suffix_names_a_comma_separated_table() {
        assert_eq!(Separator::localized("Unit_Explanation_ja.csv"), Separator::Comma);
        assert_eq!(Separator::localized("Unit_Explanation_en.csv"), Separator::Pipe);
        assert_eq!(Separator::localized("Unit_Explanation_th.csv"), Separator::Pipe);
        // A localized table shipped with no suffix at all is still bar-separated.
        assert_eq!(Separator::localized("matatabi_Popup.csv"), Separator::Pipe);
        assert_eq!(Separator::localized("TitleMessage"), Separator::Pipe);
        // "ja" has to be the suffix, not merely the tail of a longer word.
        assert_eq!(Separator::localized("jinja_level.csv"), Separator::Pipe);
    }

    #[test]
    fn detection_falls_back_to_a_comma() {
        assert_eq!(resolve(None, "one whole name"), ',');
        assert_eq!(Separator::ALL.map(Separator::char), ['|', '\t', ',']);
    }

    #[test]
    fn lookup_reads_the_column_of_the_stated_delimiter() {
        let table = b"1|Soracte, mentor mental|Un mentor.";
        assert_eq!(
            lookup(table, "1", 0, 1, Some(Separator::Pipe)).as_deref(),
            Some("Soracte, mentor mental"),
        );
    }
}
