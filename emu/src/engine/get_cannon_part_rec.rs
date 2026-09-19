use crate::Fault;

use super::{AppContext, CannonPart, get_cannon_part_id};

pub fn get_cannon_part_rec(ctx: &mut AppContext) -> Result<&mut CannonPart, Fault> {
    let id = get_cannon_part_id(ctx)?;

    Ok(ctx.cannon_parts.entry(id).or_default())
}
