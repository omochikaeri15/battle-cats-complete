use crate::Fault;

use super::{AppContext, CannonPart, get_foundation_part_id};

pub fn get_foundation_part_rec(ctx: &mut AppContext) -> Result<&mut CannonPart, Fault> {
    let part_id = get_foundation_part_id(ctx)?;

    Ok(ctx.cannon_parts.entry(part_id).or_default())
}
