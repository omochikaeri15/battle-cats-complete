use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, EnigmaActive, call_rng, enigma_pick_by_medals, enigma_pick_by_table,
    event_schedule_active, get_stage_record, map_index_of_map_id, map_type_of_map_id, now_seconds,
};

pub fn enigma_roll(ctx: &mut AppContext) -> Result<i32, Fault> {
    if ctx.enigma.active.len() > 3 {
        return Ok(0);
    }

    let now = now_seconds(ctx)?;
    let mut eligible: BTreeMap<i32, bool> = BTreeMap::new();

    for ids in event_schedule_active(ctx, now)? {
        for id in ids {
            if (id.wrapping_sub(0x6590) as u32) > 0x3e7 {
                continue;
            }

            let group = id.wrapping_sub(0x6590);

            let Some(map_stage) = ctx
                .enigma
                .groups
                .get(group as i64 as usize)
                .map(|entry| entry.map_stage)
            else {
                continue;
            };
            let map = map_stage / 100;
            let cleared = get_stage_record(
                ctx,
                map_type_of_map_id(map),
                map_index_of_map_id(map),
                map_stage.wrapping_sub(map.wrapping_mul(100)),
                0,
                0,
            )?;

            if cleared > 0 {
                *eligible.entry(group).or_insert(false) = true;
            }
        }
    }

    let mut excluded: BTreeMap<i32, bool> = BTreeMap::new();

    for active in &ctx.enigma.active {
        *excluded.entry(active.map).or_insert(false) = true;
    }

    let mut picked = enigma_pick_by_table(ctx, eligible.clone(), excluded.clone())?;

    if picked == -1 {
        picked = enigma_pick_by_medals(ctx, eligible, excluded.clone())?;

        if picked == -1 {
            return Ok(0);
        }
    }

    let group = picked as i64 as usize;

    if ctx.enigma.groups.len() <= group {
        return Err(Fault::out_of_range());
    }

    let mut total = 0i32;
    let mut slot = 0usize;

    loop {
        let entry = ctx
            .enigma
            .groups
            .get(group)
            .ok_or(Fault::out_of_range())?;

        if slot >= entry.stage_ids.len() {
            break;
        }

        let id = *entry
            .stage_ids
            .get(slot)
            .ok_or(Fault::out_of_range())?;

        if !*excluded.entry(id).or_insert(false) {
            let entry = ctx
                .enigma
                .groups
                .get(group)
                .ok_or(Fault::out_of_range())?;

            total = total.wrapping_add(*entry.weights.get(slot).ok_or(Fault::index_out_of_range(slot as i64, entry.weights.len() as i64))?);
        }

        slot += 1;
    }

    let roll = call_rng(ctx, total);
    let result = (picked & !0xff) | 1;
    let mut sum = 0i32;
    let mut slot = 0usize;

    loop {
        let entry = ctx
            .enigma
            .groups
            .get(group)
            .ok_or(Fault::out_of_range())?;

        if slot >= entry.stage_ids.len() {
            return Ok(result);
        }

        let id = *entry
            .stage_ids
            .get(slot)
            .ok_or(Fault::out_of_range())?;

        if !*excluded.entry(id).or_insert(false) {
            let entry = ctx
                .enigma
                .groups
                .get(group)
                .ok_or(Fault::out_of_range())?;

            sum = sum.wrapping_add(*entry.weights.get(slot).ok_or(Fault::index_out_of_range(slot as i64, entry.weights.len() as i64))?);

            if roll < sum {
                let map = *ctx
                    .enigma
                    .groups
                    .get(group)
                    .ok_or(Fault::out_of_range())?
                    .stage_ids
                    .get(slot)
                    .ok_or(Fault::out_of_range())?;
                let time = now_seconds(ctx)?;

                ctx.enigma.active.push(EnigmaActive {
                    state: 0,
                    time,
                    group: picked,
                    map,
                });

                ctx.meta()
                    .ok_or(Fault::host_missing())?
                    .enigma_opened(map, time);

                return Ok(result);
            }
        }

        slot += 1;
    }
}
