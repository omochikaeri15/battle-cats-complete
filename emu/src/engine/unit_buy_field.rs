use crate::{operation, Fault};

use super::UnitBuy;

pub fn unit_buy_field(row: &[u8], index: usize) -> Result<i32, Fault> {
    Ok(operation::xor_row_decode(row, UnitBuy::KEY / 4, index).ok_or(Fault::IndexOutOfRange { site: "unit_buy_field", index: index as i64, limit: (UnitBuy::KEY / 4) as i64 })? as i32)
}
