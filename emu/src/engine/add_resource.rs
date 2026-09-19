use crate::{operation, Fault};

use super::{get_item_count, set_item_count, AppContext, ItemDefinition};

pub fn add_resource(ctx: &mut AppContext, item: i32, amount: i32, flag: u8) -> Result<(), Fault> {
    let record = ((item as i64) << 6) as usize;
    let redirect = ctx.i32_at(AppContext::ITEM_DEFINITIONS.wrapping_add(record).wrapping_add(ItemDefinition::REDIRECT))?;

    if redirect == -1 {
        let count = get_item_count(ctx, item)?.wrapping_add(amount);

        return set_item_count(ctx, item, count, flag);
    }

    let current = get_item_count(ctx, redirect)?;

    let multiplier = if ctx.i32_at(AppContext::ITEM_DEFINITIONS.wrapping_add(record).wrapping_add(ItemDefinition::REDIRECT))? == -1 {
        1
    } else {
        operation::xor_cell_decode(&ctx.block_at::<8>(AppContext::ITEM_REDIRECT_SCALES.wrapping_add(record))?) as i32
    };

    set_item_count(ctx, redirect, multiplier.wrapping_mul(amount).wrapping_add(current), flag)
}
