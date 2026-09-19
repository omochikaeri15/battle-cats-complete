use crate::Fault;

use super::{read_flag, AppContext, Entity};

pub fn get_unit_name(ctx: &AppContext, faction: i32, slot: i32) -> Result<Vec<u8>, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? != 0 && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? == 1 {
        return Ok(b"\xe5\x9f\x8e".to_vec());
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? == 0 {
        return Ok(b"None".to_vec());
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        let row = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? as i64;

        return Ok(ctx.enemy_names.get(row as usize).cloned().unwrap_or_default());
    }

    let row = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))? as i64;
    let unit_id = row.wrapping_sub(2);
    let form = ctx.i32_at((row * 4 + AppContext::FACTION_1_UNIT_FORMS as i64) as usize)? as i64;

    Ok(ctx.cat_names.get(unit_id as usize).and_then(|forms| forms.get(form as usize)).map(|record| record[0].clone()).unwrap_or_default())
}
