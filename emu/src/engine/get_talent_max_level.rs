use crate::Fault;

use super::AppContext;

pub fn get_talent_max_level(
    ctx: &mut AppContext,
    _faction: i32,
    unit_id: i32,
    abil: i32,
) -> Result<i32, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(0);
    }

    let mut slot = 0usize;

    while slot < 8 {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

        if definition[slot * 0xe + 1] == abil {
            return Ok(definition[slot * 0xe + 2]);
        }

        slot += 1;
    }

    Ok(0)
}
