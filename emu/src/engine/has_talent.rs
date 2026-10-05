use crate::Fault;

use super::{AppContext, get_scene_id, has_fixed_lineup};

pub fn has_talent(
    ctx: &mut AppContext,
    _faction: i32,
    unit_id: i32,
    form: i32,
    abil: i32,
) -> Result<bool, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(false);
    }

    if form < 2 {
        return Ok(false);
    }

    if get_scene_id(ctx)? == 0x12c && has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(false);
    }

    let mut talent_slot = 0;

    loop {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());

        if definition[talent_slot * 0xe + 1] == abil {
            let levels = ctx.talent_levels.entry(unit_id).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());
            let level = levels.entry(definition[talent_slot * 0xe + 1]).or_default();

            if *level > 0 {
                return Ok(true);
            }
        }

        talent_slot += 1;

        if talent_slot == ctx.limits.talent_groups as usize {
            return Ok(false);
        }
    }
}
