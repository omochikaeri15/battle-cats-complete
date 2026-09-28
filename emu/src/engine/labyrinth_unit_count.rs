use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{AppContext, get_unit_rarity};

pub fn labyrinth_unit_count(ctx: &mut AppContext, rarity: i32) -> Result<i32, Fault> {
    let mut deck: BTreeMap<i32, bool> = BTreeMap::new();

    for slot in 0..10usize {
        let row = ctx.bytes_from(AppContext::BATTLE_LINEUP)?;
        let unit = (ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as i32).wrapping_sub(2);

        if unit < 0 {
            continue;
        }

        let row = ctx.bytes_from(AppContext::BATTLE_LINEUP)?;
        let unit = (ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))? as i32).wrapping_sub(2);

        *deck.entry(unit).or_default() = true;
    }

    let mut count = 0i32;

    for unit in 0..0x36cusize {
        let row = ctx.bytes_from(AppContext::UNITS_OWNED)?;
        let owned = ops::xor_row_decode(row, 0x36c, unit).ok_or(Fault::index_out_of_range(unit as i64, 0x36c))?;

        if owned == 0 {
            continue;
        }

        if rarity != -1 && get_unit_rarity(ctx, unit as i32)? != rarity {
            continue;
        }

        if *ctx.labyrinth_units_used.entry(unit as i32).or_default() {
            continue;
        }

        count = count.wrapping_add(i32::from(!*deck.entry(unit as i32).or_default()));
    }

    Ok(count)
}
