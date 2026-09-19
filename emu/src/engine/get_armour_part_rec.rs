use crate::Fault;

use super::{get_foundation_part_id, AppContext, CannonPart};

pub fn get_armour_part_rec(ctx: &mut AppContext) -> Result<&mut CannonPart, Fault> {
    let part_id = get_foundation_part_id(ctx)?;

    Ok(ctx.cannon_parts.entry(part_id).or_default())
}
