use std::collections::BTreeMap;

use crate::Fault;

use super::{get_global_map_id, get_stage_index, get_star_level, AppContext};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct FixedLineupRow {
    pub map_id: i32,
    pub level: i32,
    pub stage: i32,
    pub name: String,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct FixedLineupUnit {
    pub unit_id: i32,
    pub unknown_1: i32,
    pub level: i32,
    pub plus_level: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct FixedLineupStore {
    pub units: Vec<FixedLineupUnit>,
    pub rows: Vec<FixedLineupRow>,
    pub hints: Vec<FixedLineupRow>,
    pub ability_levels: BTreeMap<i32, i32>,
}

pub fn has_fixed_lineup(ctx: &mut AppContext, mut map_id: i32, mut stage: i32, mut level: i32) -> Result<bool, Fault> {
    if map_id == -1 {
        map_id = get_global_map_id(ctx, 0)?;
        stage = get_stage_index(ctx)?;
        level = get_star_level(ctx)?;

        if map_id == -1 {
            map_id = get_global_map_id(ctx, 0)?;
            stage = get_stage_index(ctx)?;
            level = get_star_level(ctx)?;
        }
    }

    for row in &ctx.fixed_lineup_store.rows {
        if row.map_id == map_id && row.stage == stage && row.level == level {
            return Ok(true);
        }
    }

    Ok(false)
}
