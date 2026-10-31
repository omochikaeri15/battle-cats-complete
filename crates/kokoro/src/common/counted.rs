use std::io::{self, Read};

pub(crate) struct Counted<'a, F: FnMut(f32)> {
    bytes: &'a [u8],
    at: usize,
    report: F,
}

impl<'a, F: FnMut(f32)> Counted<'a, F> {
    pub(crate) fn new(bytes: &'a [u8], report: F) -> Self {
        Self { bytes, at: 0, report }
    }
}

impl<F: FnMut(f32)> Read for Counted<'_, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rest = self.bytes.get(self.at..).unwrap_or_default();
        let taken = rest.len().min(buf.len());

        if let (Some(into), Some(from)) = (buf.get_mut(..taken), rest.get(..taken)) {
            into.copy_from_slice(from);
        }

        self.at += taken;
        (self.report)(self.at as f32 / self.bytes.len().max(1) as f32);

        Ok(taken)
    }
}
