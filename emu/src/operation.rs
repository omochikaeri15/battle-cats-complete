mod xor_cell_decode;
mod xor_row_decode;

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
