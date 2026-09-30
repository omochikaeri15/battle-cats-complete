//! The engine's own reader for its comma delimited animation files.
//!
//! The three animation loaders share one text stream, and its behaviour decides
//! what a malformed file parses to. It addresses a row by column index rather
//! than by a moving cursor, reads a missing column as zero rather than as an
//! error, refills its cell buffer only when a line was actually there, and takes
//! a count at its word however many rows follow it. A parser reproducing those
//! rules agrees with the engine on files no sensible parser would agree on.

use std::borrow::Cow;

/// The delimiter the animation loaders split a row on.
const DELIMITER: char = ',';

/// The line endings the engine trims from the tail of a line.
const ENDINGS: [char; 2] = ['\r', '\n'];

/// The characters the engine's cell reader skips before a sign or a digit.
const BLANKS: [char; 6] = [' ', '\t', '\n', '\u{b}', '\u{c}', '\r'];

/// The most rows a declared count may ask for.
const COUNT_CAP: usize = 1 << 16;

/// A reader over one of the engine's comma delimited animation files.
///
/// A row read past the end of the file leaves the previous row's cells in place,
/// because the engine refills its cell buffer only once it has a line to fill it
/// from. A loader reading a fixed number of rows out of a file that runs out
/// therefore repeats the last row it managed to read.
#[derive(Clone, Debug, Default)]
pub struct Reader<'a> {
    rest: &'a str,
    cells: Vec<&'a str>,
    values: Vec<Cow<'a, str>>,
}

impl<'a> Reader<'a> {
    /// Opens a reader over a file's text.
    ///
    /// # Arguments
    /// * `text` - The whole text of the file, already decoded.
    ///
    /// # Returns
    /// A `Reader` positioned at the first line.
    pub fn new(text: &'a str) -> Self {
        Self { rest: text, cells: Vec::new(), values: Vec::new() }
    }

    /// Reads one raw line, which is how a loader discards its header lines.
    ///
    /// A blank line is a line like any other, and a line made up entirely of
    /// line endings keeps them, which is where the engine's own trim gives up.
    ///
    /// # Returns
    /// An `Option` holding the line without its trailing line endings, or `None`
    /// once the file is exhausted.
    pub fn line(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() { return None; }

        let (line, rest) = self.rest.split_once('\n').unwrap_or((self.rest, ""));
        self.rest = rest;

        let trimmed = line.trim_end_matches(ENDINGS);

        Some(if trimmed.is_empty() { line } else { trimmed })
    }

    /// Reads one row into the cell table, which a read past the end leaves untouched.
    ///
    /// A line's final field is dropped where it is empty, so a row written with a
    /// trailing delimiter holds the same cells as one written without.
    pub fn row(&mut self) {
        let Some(line) = self.line() else { return };

        self.cells.clear();
        self.values.clear();

        if line.is_empty() { return; }

        for field in line.strip_suffix(DELIMITER).unwrap_or(line).split(DELIMITER) {
            self.cells.push(field);
            self.values.push(scalar(field));
        }
    }

    /// Returns the current row's cells exactly as the file writes them.
    ///
    /// # Returns
    /// A slice holding one entry per cell, which is empty before the first row
    /// is read.
    pub fn cells(&self) -> &[&'a str] {
        &self.cells
    }

    /// Returns the current row's cells as the text a column table reads them from.
    ///
    /// Each cell is reduced to the integer the engine takes from it, so a column
    /// table applied to these agrees with the engine on a cell carrying trailing
    /// text or no digits at all.
    ///
    /// # Returns
    /// A `Vec<&str>` holding one entry per cell, parallel to [`Reader::cells`].
    pub fn scalars(&self) -> Vec<&str> {
        self.values.iter().map(Cow::as_ref).collect()
    }

    /// Returns the integer one cell of the current row names.
    ///
    /// # Arguments
    /// * `index` - The zero based position of the cell within the row.
    ///
    /// # Returns
    /// An `i32` holding the value, which is zero where the row does not reach
    /// that position.
    pub fn value(&self, index: usize) -> i32 {
        self.values.get(index).and_then(|text| text.parse().ok()).unwrap_or(0)
    }
}

/// Returns the integer the engine's cell reader takes from a piece of text.
///
/// Reading stops at the first character that is not part of a decimal integer,
/// so text carrying a number in front of anything else contributes that number
/// and text carrying no digits at all contributes zero. A run of digits too long
/// for an `i32` saturates and then wraps, as the engine's own conversion does.
///
/// # Arguments
/// * `text` - The cell or line to read.
///
/// # Returns
/// An `i32` holding the value.
pub fn cell_value(text: &str) -> i32 {
    scalar(text).parse().unwrap_or(0)
}

/// Turns a count a file declares into the number of rows to read after it.
///
/// The engine sign extends the count and compares it unsigned, so a negative
/// count reads as enormous, takes the grow path, and throws out of the vector
/// helper uncaught. A merely large one reaches the allocation instead and
/// exhausts the heap. Both kill the process, which a library reports rather than
/// reproduces.
///
/// # Arguments
/// * `count` - The count the file declares.
///
/// # Returns
/// An `Option` holding the number of rows to read, or `None` for a count the
/// engine would abort on.
pub fn declared_rows(count: i32) -> Option<usize> {
    usize::try_from(count).ok().filter(|count| *count <= COUNT_CAP)
}

/// The text the engine's cell reader consumes, as a value an `i32` can hold.
fn scalar(cell: &str) -> Cow<'_, str> {
    let text = cell.trim_start_matches(BLANKS);
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    let digits = body.len() - body.trim_start_matches(|character: char| character.is_ascii_digit()).len();

    if digits == 0 { return Cow::Borrowed(""); }

    let read = &text[..text.len() - body.len() + digits];

    if read.parse::<i32>().is_ok() { return Cow::Borrowed(read); }

    let saturated = read.parse::<i64>()
        .unwrap_or(if read.starts_with('-') { i64::MIN } else { i64::MAX });

    Cow::Owned((saturated as i32).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_carry_the_blank_ones_and_stop_at_the_end() {
        let mut reader = Reader::new("first\r\n\nlast");

        assert_eq!(reader.line(), Some("first"));
        assert_eq!(reader.line(), Some(""));
        assert_eq!(reader.line(), Some("last"));
        assert_eq!(reader.line(), None);

        assert_eq!(Reader::new("").line(), None);
        assert_eq!(Reader::new("\n").line(), Some(""));
        assert_eq!(Reader::new("\r").line(), Some("\r"));
    }

    #[test]
    fn a_trailing_delimiter_adds_no_cell() {
        let mut reader = Reader::new("1,2,\n1,2\n,\n\n");

        reader.row();
        assert_eq!(reader.cells(), ["1", "2"]);

        reader.row();
        assert_eq!(reader.cells(), ["1", "2"]);

        reader.row();
        assert_eq!(reader.cells(), [""]);

        reader.row();
        assert_eq!(reader.cells(), [] as [&str; 0]);
    }

    #[test]
    fn a_row_past_the_end_leaves_the_last_one_in_place() {
        let mut reader = Reader::new("7,8");

        reader.row();
        assert_eq!(reader.value(0), 7);

        reader.row();
        assert_eq!(reader.value(0), 7);
        assert_eq!(reader.value(1), 8);
        assert_eq!(reader.value(2), 0);
    }

    #[test]
    fn a_cell_contributes_the_number_in_front_of_its_text() {
        let mut reader = Reader::new(" -12abc,x,3.75,+4,,99999999999");

        reader.row();

        assert_eq!(reader.value(0), -12);
        assert_eq!(reader.value(1), 0);
        assert_eq!(reader.value(2), 3);
        assert_eq!(reader.value(3), 4);
        assert_eq!(reader.value(4), 0);
        assert_eq!(reader.value(5), 99_999_999_999_i64 as i32);

        assert_eq!(cell_value(" 42 tail"), 42);
        assert_eq!(cell_value("tail"), 0);
    }

    #[test]
    fn a_count_the_engine_aborts_on_is_refused() {
        assert_eq!(declared_rows(0), Some(0));
        assert_eq!(declared_rows(12), Some(12));
        assert_eq!(declared_rows(-1), None);
        assert_eq!(declared_rows(i32::MAX), None);
    }
}
