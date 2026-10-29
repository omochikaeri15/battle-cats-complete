//! Sanitization and structural probing of raw delimited text files.
//!
//! The engine's data files vary in encoding, line ending, and delimiter between
//! regions and versions, so every parser normalizes its input through these
//! helpers before reading any columns.
//!
//! Which delimiter a file uses is a property of the file rather than of its
//! bytes: a localized table is pipe delimited in most languages and comma
//! delimited in others, and a name carrying a comma makes the two
//! indistinguishable from the content alone. A parser therefore takes the
//! delimiter from its caller, and falls back to [`Separator::detect`] only when
//! the caller states none.

use serde::{Deserialize, Serialize};

/// The language suffix the Japanese text tables carry.
const JAPANESE: &str = "ja";

/// A delimiter one of the engine's delimited text files separates its columns with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Separator {
    /// A vertical bar, used by most localized text tables.
    Pipe,
    /// A horizontal tab.
    Tab,
    /// A comma, used by the mechanical tables and by the Japanese text tables.
    Comma,
}

impl Separator {
    /// Every delimiter the engine's delimited files are written with.
    pub const ALL: [Self; 3] = [Self::Pipe, Self::Tab, Self::Comma];

    /// The delimiter a payload appears to be written with.
    ///
    /// A bar anywhere in the payload wins outright, and otherwise the first tab
    /// or comma decides. A file whose first field carries a comma before its
    /// first real delimiter is therefore misread, which is why a caller that
    /// knows the file should state the delimiter instead.
    ///
    /// # Arguments
    /// * `text` - The sanitized text to probe.
    ///
    /// # Returns
    /// An `Option` holding the delimiter the payload appears to use, or `None`
    /// when it carries none of them.
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

    /// The delimiter a localized text table with this file name is written with.
    ///
    /// The Japanese tables separate their columns with a comma and write any
    /// comma inside a field as a wide one; every other language separates with a
    /// bar so that a field may carry an ordinary comma. Only a `_ja` suffix
    /// names a Japanese table, so a table shipped without a language suffix is
    /// bar-separated like the rest.
    ///
    /// This reads the name alone and never the payload, which is what makes it
    /// safe on a table whose first field legitimately carries a comma. Reach for
    /// [`Separator::detect`] only when the name is unknown.
    ///
    /// # Arguments
    /// * `name` - The file name, with or without its extension.
    ///
    /// # Returns
    /// A `Separator` holding the delimiter that table separates its columns with.
    pub fn localized(name: &str) -> Self {
        let stem = name.rsplit_once('.').map_or(name, |(head, _)| head);
        let japanese = stem.rsplit_once('_').is_some_and(|(_, code)| code == JAPANESE);

        if japanese { Self::Comma } else { Self::Pipe }
    }

    /// The character this delimiter is written as.
    ///
    /// # Returns
    /// A `char` holding the delimiter.
    pub const fn char(self) -> char {
        match self {
            Self::Pipe => '|',
            Self::Tab => '\t',
            Self::Comma => ',',
        }
    }

    /// Splits one line into its columns on this delimiter.
    ///
    /// # Arguments
    /// * `line` - The line to split.
    ///
    /// # Returns
    /// An iterator over the line's columns, exactly as they are written.
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

/// Converts raw bytes to a string, dropping byte-order marks and null characters
/// and normalizing line endings.
///
/// # Arguments
/// * `bytes` - The raw bytes to sanitize.
///
/// # Returns
/// A `String` safe to traverse with the standard text utilities.
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

/// How `<br>` tags are substituted when stripping HTML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakHandling {
    /// Replaces `<br>` with a single space, collapsing runs.
    Space,
    /// Replaces `<br>` with a newline character (`\n`).
    Newline,
    /// Completely deletes the `<br>` tag without injecting any substitute character.
    Delete,
}

/// Strips HTML tags from a string, handling `<br>` by the given strategy.
///
/// # Arguments
/// * `input` - The raw text to strip.
/// * `handling` - How `<br>` elements should be substituted.
///
/// # Returns
/// A `String` with tags removed and breaks substituted.
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

/// Finds a row by one column's value and returns another column from it.
///
/// # Arguments
/// * `data` - The raw bytes of the table to search.
/// * `key` - The value to match against `search_col`.
/// * `search_col` - The zero-indexed column to match on.
/// * `target_col` - The zero-indexed column to extract from the matched row.
/// * `separator` - The delimiter the table is written with, or `None` to detect it from the content.
///
/// # Returns
/// An `Option` containing the trimmed value, or `None` if no row matched.
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
