use std::borrow::Cow;

const DELIMITER: char = ',';

const ENDINGS: [char; 2] = ['\r', '\n'];

const BLANKS: [char; 6] = [' ', '\t', '\n', '\u{b}', '\u{c}', '\r'];

const COUNT_CAP: usize = 1 << 16;

#[derive(Clone, Debug, Default)]
pub struct Reader<'a> {
    rest: &'a str,
    cells: Vec<&'a str>,
    values: Vec<Cow<'a, str>>,
}

impl<'a> Reader<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { rest: text, cells: Vec::new(), values: Vec::new() }
    }

    pub fn line(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() { return None; }

        let (line, rest) = self.rest.split_once('\n').unwrap_or((self.rest, ""));
        self.rest = rest;

        let trimmed = line.trim_end_matches(ENDINGS);

        Some(if trimmed.is_empty() { line } else { trimmed })
    }

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

    pub fn cells(&self) -> &[&'a str] {
        &self.cells
    }

    pub fn scalars(&self) -> Vec<&str> {
        self.values.iter().map(Cow::as_ref).collect()
    }

    pub fn value(&self, index: usize) -> i32 {
        self.values.get(index).and_then(|text| text.parse().ok()).unwrap_or(0)
    }
}

pub fn cell_value(text: &str) -> i32 {
    scalar(text).parse().unwrap_or(0)
}

pub fn declared_rows(count: i32) -> Option<usize> {
    usize::try_from(count).ok().filter(|count| *count <= COUNT_CAP)
}

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
