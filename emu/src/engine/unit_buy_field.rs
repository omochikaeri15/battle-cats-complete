use crate::{Fault, operation};

use super::UnitBuy;

pub fn unit_buy_field(row: &[u8], index: usize) -> Result<i32, Fault> {
    Ok(
        operation::xor_row_decode(row, UnitBuy::KEY / 4, index).ok_or(Fault::index_out_of_range(index as i64, (UnitBuy::KEY / 4) as i64))? as i32,
    )
}
