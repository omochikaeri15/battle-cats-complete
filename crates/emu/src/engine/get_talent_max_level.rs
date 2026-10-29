use crate::Fault;

use super::AppContext;

pub fn get_talent_max_level(
    ctx: &mut AppContext,
    _faction: i32,
    unit_id: i32,
    ability: i32,
) -> Result<i32, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(0);
    }

    let mut slot = 0usize;

    while slot < ctx.limits.talent_groups as usize {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert_with(|| ctx.limits.talent_row());

        if definition[slot * 0xe + 1] == ability {
            return Ok(definition[slot * 0xe + 2]);
        }

        slot += 1;
    }

    Ok(0)
}
