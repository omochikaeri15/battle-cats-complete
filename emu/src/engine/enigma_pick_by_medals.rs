use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, call_rng, medal_awarded};

pub fn enigma_pick_by_medals(
    ctx: &mut AppContext,
    mut eligible: BTreeMap<i32, bool>,
    mut excluded: BTreeMap<i32, bool>,
) -> Result<i32, Fault> {
    if ctx.enigma.medals.is_empty() {
        return Ok(-1);
    }

    let mut owned = 0i32;
    let mut index = 0usize;

    loop {
        let medal = *ctx
            .enigma
            .medals
            .get(index)
            .ok_or(Fault::out_of_range())?;

        owned = owned.wrapping_add(medal_awarded(ctx, medal) as i32);
        index += 1;

        if index >= ctx.enigma.medals.len() {
            break;
        }
    }

    if owned == 0 || (ctx.enigma.tiers.len() as u64) < owned as u32 as u64 {
        return Ok(-1);
    }

    if owned > ctx.enigma.medal_count {
        ctx.enigma.last_threshold = 0;
    }

    ctx.enigma.medal_count = owned;

    let tier = owned.wrapping_sub(1) as i64 as usize;

    if ctx.enigma.pools.len() <= tier {
        return Ok(-1);
    }

    let mut candidates: BTreeMap<i32, bool> = BTreeMap::new();
    let mut index = 0usize;

    loop {
        let pool = ctx
            .enigma
            .pools
            .get(tier)
            .ok_or(Fault::out_of_range())?;

        if index >= pool.groups.len() {
            break;
        }

        let group = *pool
            .groups
            .get(index)
            .ok_or(Fault::out_of_range())?;

        if *eligible.entry(group).or_insert(false) {
            let mut slot = 0usize;

            loop {
                let stages = &ctx
                    .enigma
                    .groups
                    .get(group as i64 as usize)
                    .ok_or(Fault::out_of_range())?
                    .stage_ids;

                if slot >= stages.len() {
                    break;
                }

                let id = *stages.get(slot).ok_or(Fault::out_of_range())?;

                if !*excluded.entry(id).or_insert(false) {
                    candidates.insert(group, true);

                    break;
                }

                slot += 1;
            }
        }

        index += 1;

        if ctx.enigma.pools.len() <= tier {
            return Err(Fault::out_of_range());
        }
    }

    if candidates.is_empty() {
        return Ok(-1);
    }

    if ctx.enigma.tiers.len() <= tier {
        return Err(Fault::out_of_range());
    }

    let mut step = 0usize;

    loop {
        let thresholds = &ctx
            .enigma
            .tiers
            .get(tier)
            .ok_or(Fault::out_of_range())?
            .thresholds;

        if step >= thresholds.len() {
            ctx.enigma.last_threshold = ctx.enigma.stamina;

            return Ok(-1);
        }

        let threshold = *thresholds
            .get(step)
            .ok_or(Fault::out_of_range())?;

        if threshold > ctx.enigma.last_threshold {
            if threshold > ctx.enigma.stamina {
                ctx.enigma.last_threshold = ctx.enigma.stamina;

                return Ok(-1);
            }

            let roll = call_rng(ctx, 0x3e8);
            let chances = &ctx
                .enigma
                .tiers
                .get(tier)
                .ok_or(Fault::out_of_range())?
                .chances;
            let chance = *chances.get(step).ok_or(Fault::index_out_of_range(step as i64, chances.len() as i64))?;

            if roll < chance {
                let mut total = 0i32;
                let mut slot = 0usize;

                loop {
                    let pool = ctx
                        .enigma
                        .pools
                        .get(tier)
                        .ok_or(Fault::out_of_range())?;

                    if slot >= pool.groups.len() {
                        break;
                    }

                    let group = *pool
                        .groups
                        .get(slot)
                        .ok_or(Fault::out_of_range())?;

                    if *candidates.entry(group).or_insert(false) {
                        let pool = ctx
                            .enigma
                            .pools
                            .get(tier)
                            .ok_or(Fault::out_of_range())?;

                        total = total.wrapping_add(*pool.weights.get(slot).ok_or(
                            Fault::index_out_of_range(slot as i64, pool.weights.len() as i64),
                        )?);
                    }

                    slot += 1;
                }

                let pick = call_rng(ctx, total);
                let mut sum = 0i32;
                let mut slot = 0usize;
                let chosen = loop {
                    let pool = ctx
                        .enigma
                        .pools
                        .get(tier)
                        .ok_or(Fault::out_of_range())?;

                    if slot >= pool.groups.len() {
                        break -1;
                    }

                    let group = *pool
                        .groups
                        .get(slot)
                        .ok_or(Fault::out_of_range())?;

                    if *candidates.entry(group).or_insert(false) {
                        let pool = ctx
                            .enigma
                            .pools
                            .get(tier)
                            .ok_or(Fault::out_of_range())?;

                        sum = sum.wrapping_add(*pool.weights.get(slot).ok_or(
                            Fault::index_out_of_range(slot as i64, pool.weights.len() as i64),
                        )?);

                        if pick < sum {
                            break group;
                        }
                    }

                    slot += 1;
                };

                ctx.enigma.stamina = 0;
                ctx.enigma.last_threshold = 0;

                return Ok(chosen);
            }
        }

        step += 1;
    }
}
