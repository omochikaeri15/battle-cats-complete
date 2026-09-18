use crate::Fault;

use super::{AppContext, Entity};

pub fn get_kb_proc_hit(ctx: &AppContext, faction: i32, slot: i32) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::KB_PROC_HIT))? != 0)
}
