use crate::Fault;

use super::{AppContext, get_scene_id, has_fixed_lineup};

pub fn get_talent_icon_state(
    ctx: &mut AppContext,
    _faction: i32,
    unit_id: i32,
    form: i32,
    abil: i32,
) -> Result<i32, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(0);
    }

    if form < 2 {
        return Ok(0);
    }

    if get_scene_id(ctx)? == 0x12c && has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(0);
    }

    let mut talent_slot = 0;

    loop {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());

        if definition[talent_slot * 0xe + 1] == abil {
            let levels = ctx.talent_levels.entry(unit_id).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());
            let level = *levels.entry(definition[talent_slot * 0xe + 1]).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());

            if level == definition[talent_slot * 0xe + 2] {
                return Ok(2);
            }
        }

        let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());

        if definition[talent_slot * 0xe + 1] == abil {
            let levels = ctx.talent_levels.entry(unit_id).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());
            let level = levels.entry(definition[talent_slot * 0xe + 1]).or_default();

            if *level > 0 {
                return Ok(1);
            }
        }

        talent_slot += 1;

        if talent_slot == ctx.limits.talent_groups as usize {
            return Ok(0);
        }
    }
}
