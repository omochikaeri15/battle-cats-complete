use crate::Fault;

use super::{get_scene_id, has_fixed_lineup, AppContext};

pub fn has_talent(ctx: &mut AppContext, _team: i32, unit_id: i32, form: i32, abil: i32) -> Result<bool, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(false);
    }

    if form < 2 {
        return Ok(false);
    }

    if get_scene_id(ctx)? == 0x12c && has_fixed_lineup(ctx, 0x468, -1, -1, -1)? {
        return Ok(false);
    }

    let mut talent_slot = 0;

    loop {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

        if definition[talent_slot * 0xe + 1] == abil {
            let levels = ctx.talent_levels.entry(unit_id).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
            let level = levels.entry(definition[talent_slot * 0xe + 1]).or_default();

            if *level > 0 {
                return Ok(true);
            }
        }

        talent_slot += 1;

        if talent_slot == 8 {
            return Ok(false);
        }
    }
}
