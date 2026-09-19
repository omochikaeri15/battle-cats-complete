use crate::Fault;

use super::{AppContext, get_global_map_id};

pub fn is_ex_option_target(ctx: &mut AppContext) -> Result<bool, Fault> {
    let mut found = false;
    let map_id = get_global_map_id(ctx, 0)?;

    for target in ctx.ex_option_targets.values() {
        found = *target == map_id;

        if found {
            break;
        }
    }

    Ok(found)
}
