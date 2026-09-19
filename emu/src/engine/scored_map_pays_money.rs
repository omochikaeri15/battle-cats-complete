use crate::Fault;

use super::{get_global_map_id, AppContext};

pub fn scored_map_pays_money(ctx: &mut AppContext) -> Result<bool, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    Ok(ctx.money_scored_maps.contains(&map_id))
}
