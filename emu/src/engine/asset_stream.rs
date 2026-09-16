pub struct Cell {
    pub at: usize,
    pub len: usize,
}

pub struct AssetStream<'a> {
    pub bytes: &'a [u8],
    pub cells: Vec<Cell>,
    pub cursor: usize,
    pub end: usize,
    pub line_delimiter: u8,
}

impl<'a> AssetStream<'a> {
    pub fn new(bytes: &'a [u8], line_delimiter: u8) -> Self {
        Self { bytes, cells: Vec::new(), cursor: 0, end: bytes.len(), line_delimiter }
    }
}
