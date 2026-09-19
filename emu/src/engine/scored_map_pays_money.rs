use crate::Fault;

use super::{AppContext, get_global_map_id};

pub fn scored_map_pays_money(ctx: &mut AppContext) -> Result<bool, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    Ok(ctx.scored_maps.contains_key(&map_id))
}
