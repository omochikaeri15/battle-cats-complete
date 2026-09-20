use crate::Fault;

use super::{AppContext, has_talent};

pub fn get_talent_value(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    abil: i32,
    param: i32,
) -> Result<i32, Fault> {
    if !has_talent(ctx, faction, unit_id, form, abil)? {
        return Ok(0);
    }

    let mut talent_slot = 0;

    loop {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

        if definition[talent_slot * 0xe + 1] == abil {
            break;
        }

        talent_slot += 1;

        if talent_slot == 8 {
            return Ok(0);
        }
    }

    let min_cell = (talent_slot as i64 * 0xe + 3 + param as i64) as usize;
    let max_cell = (talent_slot as i64 * 0xe + 7 + param as i64) as usize;

    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

    if definition[talent_slot * 0xe + 2] <= 1 {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

        return definition
            .get(min_cell)
            .copied()
            .ok_or(Fault::index_out_of_range(param as i64, 4));
    }

    let levels = ctx.talent_levels.entry(unit_id).or_default();
    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
    let level = *levels.entry(definition[talent_slot * 0xe + 1]).or_default();

    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
    let min = *definition.get(min_cell).ok_or(Fault::index_out_of_range(param as i64, 4))?;

    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
    let max = *definition.get(max_cell).ok_or(Fault::index_out_of_range(param as i64, 4))?;

    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
    let span = max.wrapping_sub(*definition.get(min_cell).ok_or(Fault::index_out_of_range(param as i64, 4))?);
    let scaled = level.wrapping_sub(1).wrapping_mul(span);

    let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
    let steps = definition[talent_slot * 0xe + 2].wrapping_sub(1);

    let quotient = (scaled as i64)
        .checked_div(steps as i64)
        .ok_or(Fault::divide_by_zero())? as i32;

    Ok(quotient.wrapping_add(min))
}
