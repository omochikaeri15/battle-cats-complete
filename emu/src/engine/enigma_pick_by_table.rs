use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, call_rng, get_global_map_id, get_stage_index};

const SITE: &str = "enigma_pick_by_table";

pub fn enigma_pick_by_table(
    ctx: &mut AppContext,
    mut eligible: BTreeMap<i32, bool>,
    mut excluded: BTreeMap<i32, bool>,
) -> Result<i32, Fault> {
    let map = get_global_map_id(ctx, 0)?;
    let stage = get_stage_index(ctx)?;

    if !ctx
        .enigma
        .table
        .get(&map)
        .is_some_and(|stages| stages.contains_key(&stage))
    {
        return Ok(-1);
    }

    let mut candidates: BTreeMap<i32, bool> = BTreeMap::new();
    let mut index = 0usize;

    loop {
        let entry = ctx
            .enigma
            .table
            .get(&map)
            .and_then(|stages| stages.get(&stage))
            .ok_or(Fault::KeyNotFound {
                site: SITE,
                key: map as i64,
            })?;

        if index >= entry.groups.len() {
            break;
        }

        let group = *entry
            .groups
            .get(index)
            .ok_or(Fault::OutOfRange { site: SITE })?;

        if *eligible.entry(group).or_insert(false) {
            let mut slot = 0usize;

            loop {
                let stages = &ctx
                    .enigma
                    .groups
                    .get(group as i64 as usize)
                    .ok_or(Fault::OutOfRange { site: SITE })?
                    .stage_ids;

                if slot >= stages.len() {
                    break;
                }

                let id = *stages.get(slot).ok_or(Fault::OutOfRange { site: SITE })?;

                if !*excluded.entry(id).or_insert(false) {
                    candidates.insert(group, true);

                    break;
                }

                slot += 1;
            }
        }

        index += 1;
    }

    if candidates.is_empty() {
        return Ok(-1);
    }

    let mut index = 0usize;

    loop {
        let entry = ctx
            .enigma
            .table
            .get(&map)
            .and_then(|stages| stages.get(&stage))
            .ok_or(Fault::KeyNotFound {
                site: SITE,
                key: map as i64,
            })?;

        if index >= entry.chances.len() {
            return Ok(-1);
        }

        let group = *entry
            .groups
            .get(index)
            .ok_or(Fault::OutOfRange { site: SITE })?;

        if *candidates.entry(group).or_insert(false) {
            let roll = call_rng(ctx, 0x64);
            let entry = ctx
                .enigma
                .table
                .get(&map)
                .and_then(|stages| stages.get(&stage))
                .ok_or(Fault::KeyNotFound {
                    site: SITE,
                    key: map as i64,
                })?;
            let chance = *entry
                .chances
                .get(index)
                .ok_or(Fault::OutOfRange { site: SITE })?;

            if roll < chance {
                return Ok(group);
            }
        }

        index += 1;
    }
}
