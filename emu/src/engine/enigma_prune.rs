use crate::Fault;

use super::{AppContext, map_one_time, map_one_time_limit, map_type_base_id, now_seconds};

const SITE: &str = "enigma_prune";

pub fn enigma_prune(ctx: &mut AppContext) -> Result<(), Fault> {
    let now = now_seconds(ctx)?;
    let mut expired: Vec<i32> = Vec::new();
    let mut index = 0usize;

    while index < ctx.enigma.active.len() {
        let active = *ctx
            .enigma
            .active
            .get(index)
            .ok_or(Fault::OutOfRange { site: SITE })?;

        if active.state == 2 {
            let expire = 'check: {
                if active.time > now {
                    break 'check true;
                }

                let slot = active.map as i64 - map_type_base_id(-0x11, 0) as i64;
                let minutes =
                    *ctx.enigma_durations
                        .get(slot as usize)
                        .ok_or(Fault::IndexOutOfRange {
                            site: SITE,
                            index: slot,
                            limit: ctx.enigma_durations.len() as i64,
                        })?;

                if now >= active.time + minutes.wrapping_mul(0x3c) as f64 {
                    break 'check true;
                }

                let map = ctx
                    .enigma
                    .active
                    .get(index)
                    .ok_or(Fault::OutOfRange { site: SITE })?
                    .map;

                if !map_one_time(ctx, map) {
                    break 'check false;
                }

                let cleared = *ctx.map_clear_counts.entry(map).or_insert(0);

                cleared >= map_one_time_limit(ctx, map)
            };

            if expire {
                expired.push(index as i32);
            }
        }

        index += 1;
    }

    while let Some(slot) = expired.pop() {
        if (slot as i64 as usize) < ctx.enigma.active.len() {
            ctx.enigma.active.remove(slot as i64 as usize);
        }
    }

    if ctx.enigma.pending_set == 0 {
        return Ok(());
    }

    let pending = ctx.enigma.pending;

    if pending.time > now {
        ctx.enigma.pending_set = 0;

        return Ok(());
    }

    let slot = pending.map as i64 - map_type_base_id(-0x11, 0) as i64;
    let minutes = *ctx
        .enigma_durations
        .get(slot as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: slot,
            limit: ctx.enigma_durations.len() as i64,
        })?;

    if now >= pending.time + minutes.wrapping_mul(0x3c) as f64 {
        ctx.enigma.pending_set = 0;

        return Ok(());
    }

    if !map_one_time(ctx, pending.map) {
        return Ok(());
    }

    if *ctx.map_clear_counts.entry(pending.map).or_insert(0) < map_one_time_limit(ctx, pending.map)
    {
        return Ok(());
    }

    ctx.enigma.pending_set = 0;

    Ok(())
}
