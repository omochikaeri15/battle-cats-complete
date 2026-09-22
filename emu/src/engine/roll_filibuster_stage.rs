use crate::Fault;

use super::{AppContext, call_rng};

pub fn roll_filibuster_stage(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [0xff])?;

    if ctx.u8_at(AppContext::MAP_NEG15_CLEARED)? != 0
        && ctx.u8_at(AppContext::MAP_NEG25_CLEARED)? != 0
    {
        return Ok(());
    }

    let stage = call_rng(ctx, 0x30) as u8;

    ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [stage])?;

    let stage = stage as i8 as i32;

    if !ctx
        .outbreak_active
        .entry(9)
        .or_default()
        .contains_key(&stage)
    {
        return Ok(());
    }

    if !*ctx
        .outbreak_active
        .entry(9)
        .or_default()
        .entry(stage)
        .or_insert(false)
    {
        return Ok(());
    }

    let mut open: Vec<i32> = Vec::new();

    for candidate in 0..0x30 {
        let flagged = ctx
            .outbreak_active
            .entry(9)
            .or_default()
            .contains_key(&candidate)
            && *ctx
                .outbreak_active
                .entry(9)
                .or_default()
                .entry(candidate)
                .or_insert(false);

        if !flagged {
            open.push(candidate);
        }
    }

    if open.len() >= 2 {
        let mut index = 0usize;

        while index + 1 < open.len() {
            let span = (open.len() - 1 - index) as u64;
            let offset = (ctx.entropy.roll() % (span + 1)) as usize;

            if offset != 0 {
                open.swap(index, index + offset);
            }

            index += 1;
        }
    }

    let chosen = open.first().map_or(0xff, |stage| *stage as u8);

    ctx.set_block_at::<1>(AppContext::INVASION_STAGE, [chosen])
}
