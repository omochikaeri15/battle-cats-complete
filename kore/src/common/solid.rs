use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read};
use std::path::Path;

pub const DEFAULT_LEVEL: i32 = 19;
pub const MIN_LEVEL: i32 = 1;
pub const MAX_LEVEL: i32 = 19;

const MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];
const FILE_MODE: u32 = 0o644;

pub type Reader = tar::Archive<zstd::Decoder<'static, BufReader<File>>>;

pub fn sniff(path: &Path) -> bool {
    let mut head = [0u8; 4];

    File::open(path).and_then(|mut file| file.read_exact(&mut head)).is_ok_and(|()| head == MAGIC)
}

pub struct Writer {
    builder: tar::Builder<zstd::Encoder<'static, BufWriter<File>>>,
}

impl Writer {
    pub fn create(out: &Path, level: i32) -> io::Result<Self> {
        let file = BufWriter::new(File::create(out)?);
        let encoder = zstd::Encoder::new(file, level.clamp(MIN_LEVEL, MAX_LEVEL))?;
        let mut builder = tar::Builder::new(encoder);

        builder.mode(tar::HeaderMode::Deterministic);

        Ok(Self { builder })
    }

    pub fn add(&mut self, name: &str, path: &Path) -> io::Result<()> {
        self.builder.append_path_with_name(path, name)
    }

    pub fn add_stream(&mut self, name: &str, size: u64, data: impl Read) -> io::Result<()> {
        let mut header = tar::Header::new_gnu();

        header.set_size(size);
        header.set_mode(FILE_MODE);
        header.set_mtime(0);
        header.set_cksum();

        self.builder.append_data(&mut header, name, data)
    }

    pub fn finish(self) -> io::Result<()> {
        let encoder = self.builder.into_inner()?;
        let mut file = encoder.finish()?;

        io::Write::flush(&mut file)
    }
}

pub fn open(path: &Path) -> io::Result<Reader> {
    let decoder = zstd::Decoder::new(File::open(path)?)?;

    Ok(tar::Archive::new(decoder))
}
