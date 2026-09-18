mod strtol;
mod xor_cell_decode;
mod xor_row_decode;

pub use strtol::{strtol, Strtol};
pub use xor_cell_decode::xor_cell_decode;
pub use xor_row_decode::xor_row_decode;

mod sealed {
    pub trait Sealed {}

    impl Sealed for i32 {}
    impl Sealed for i64 {}
}

pub trait Truncating: sealed::Sealed + Copy + core::ops::Div<Output = Self> + From<i16> {}

impl Truncating for i32 {}
impl Truncating for i64 {}

pub fn slli_epi32(lanes: [u8; 16], count: u32) -> [u8; 16] {
    let mut out = [0u8; 16];

    for dword in 0..4 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&lanes[dword * 4..dword * 4 + 4]);
        out[dword * 4..dword * 4 + 4].copy_from_slice(&(i32::from_le_bytes(word) << count).to_le_bytes());
    }

    out
}

pub fn blend_epi16(a: [u8; 16], b: [u8; 16], mask: u8) -> [u8; 16] {
    let mut out = a;

    for lane in 0..8 {
        if mask & (1 << lane) != 0 {
            out[lane * 2] = b[lane * 2];
            out[lane * 2 + 1] = b[lane * 2 + 1];
        }
    }

    out
}

pub fn div_5<T: Truncating>(x: T) -> T {
    x / T::from(5)
}

pub fn div_10<T: Truncating>(x: T) -> T {
    x / T::from(10)
}

pub fn div_100<T: Truncating>(x: T) -> T {
    x / T::from(100)
}

pub fn div_300<T: Truncating>(x: T) -> T {
    x / T::from(300)
}

pub fn div_500<T: Truncating>(x: T) -> T {
    x / T::from(500)
}

pub fn div_1000<T: Truncating>(x: T) -> T {
    x / T::from(1000)
}

pub fn div_10000<T: Truncating>(x: T) -> T {
    x / T::from(10000)
}
