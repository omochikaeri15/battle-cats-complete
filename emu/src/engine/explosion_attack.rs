use crate::Fault;

use super::{AppContext, cat_attack_dispatch, enemy_attack_dispatch};

pub fn explosion_attack(
    ctx: &mut AppContext,
    event_index: i32,
    target: i32,
    attack: i32,
    dmg_scale: i32,
) -> Result<bool, Fault> {
    let event = ctx
        .explosion_events
        .get(event_index as i64 as usize)
        .ok_or(Fault::index_out_of_range(event_index as i64, ctx.explosion_events.len() as i64))?;
    let slot = event.slot;
    let proc_flags = event.proc_flags;

    if event.faction != 0 {
        return enemy_attack_dispatch(
            ctx,
            3,
            slot,
            target,
            attack,
            0,
            proc_flags[0],
            proc_flags[8],
            proc_flags[1],
            proc_flags[2],
            proc_flags[3],
            proc_flags[4],
            proc_flags[5],
            proc_flags[6],
            proc_flags[7],
            proc_flags[9],
            proc_flags[10],
            proc_flags[11],
            dmg_scale,
        );
    }

    let metal_killer_pct = event.metal_killer_pct;

    cat_attack_dispatch(
        ctx,
        3,
        slot,
        target,
        attack,
        0,
        proc_flags[0],
        proc_flags[8],
        proc_flags[1],
        proc_flags[2],
        proc_flags[3],
        proc_flags[4],
        proc_flags[5],
        proc_flags[6],
        proc_flags[7],
        proc_flags[10],
        metal_killer_pct,
        dmg_scale,
    )
}
