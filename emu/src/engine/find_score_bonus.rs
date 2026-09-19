use std::collections::BTreeMap;

use crate::Fault;

use super::{get_global_map_id, AppContext};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ScoredMap {
    pub bonuses: BTreeMap<i32, Vec<i32>>,
}

pub fn find_score_bonus(ctx: &mut AppContext, kind: i32) -> Result<Option<&Vec<i32>>, Fault> {
    let map_id = get_global_map_id(ctx, 0)?;

    if !ctx.scored_maps.contains_key(&map_id) {
        return Ok(None);
    }

    if !ctx.scored_maps.entry(map_id).or_default().bonuses.contains_key(&kind) {
        return Ok(None);
    }

    Ok(Some(ctx.scored_maps.entry(map_id).or_default().bonuses.entry(kind).or_default()))
}
