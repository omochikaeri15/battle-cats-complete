use crate::Fault;

use super::AppContext;

const SITE: &str = "get_altar_level_cap";

pub fn get_altar_level_cap(ctx: &AppContext, enemy_id: i32) -> Result<i32, Fault> {
    if !ctx.altar_level_caps.contains_key(&enemy_id) {
        return Ok(-1);
    }

    if !ctx.altar_unsealed.contains_key(&enemy_id) {
        return Ok(-1);
    }

    if *ctx
        .altar_unsealed
        .get(&enemy_id)
        .ok_or(Fault::KeyNotFound {
            site: SITE,
            key: enemy_id as i64,
        })?
    {
        return Ok(-1);
    }

    ctx.altar_level_caps
        .get(&enemy_id)
        .copied()
        .ok_or(Fault::KeyNotFound {
            site: SITE,
            key: enemy_id as i64,
        })
}
