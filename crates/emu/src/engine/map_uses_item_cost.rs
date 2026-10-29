use crate::Fault;

use super::{AppContext, get_map_index, get_map_type, map_item_cost_flag};

pub fn map_uses_item_cost(ctx: &mut AppContext) -> Result<bool, Fault> {
    let map_type = get_map_type(ctx, 0)?;
    let map_index = get_map_index(ctx, 0)?;

    map_item_cost_flag(ctx, map_type, map_index)
}
