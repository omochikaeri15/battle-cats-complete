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

pub fn div_12<T: Truncating>(x: T) -> T {
    x / T::from(12)
}

pub fn div_20<T: Truncating>(x: T) -> T {
    x / T::from(20)
}

pub fn div_24<T: Truncating>(x: T) -> T {
    x / T::from(24)
}

pub fn div_48<T: Truncating>(x: T) -> T {
    x / T::from(48)
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

pub fn cvttsd2si(x: f64) -> i32 {
    if x > -2147483649.0 && x < 2147483648.0 {
        x as i32
    } else {
        i32::MIN
    }
}

pub fn div_wide(dividend: i64, divisor: i64) -> Option<i64> {
    if (dividend | divisor) as u64 >> 0x20 == 0 {
        return (dividend as u32).checked_div(divisor as u32).map(i64::from);
    }

    dividend.checked_div(divisor)
}

pub fn idiv(dividend: i32, divisor: i32) -> Option<i32> {
    dividend.checked_div(divisor)
}

pub fn irem(dividend: i32, divisor: i32) -> Option<i32> {
    dividend.checked_rem(divisor)
}

pub fn div_neg_100<T: Truncating>(x: T) -> T {
    x / T::from(-100)
}
