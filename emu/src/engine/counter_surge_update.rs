use crate::Fault;

use super::{abs_i32, call_rng, get_anim_len, get_metal_killer_pct, get_pos_x, get_setting, play_sound, roll_procs, sound_manager, AppContext, SurgeEvent};

pub fn counter_surge_update(ctx: &mut AppContext) -> Result<(), Fault> {
    const SITE: &str = "counter_surge_update";

    let mut index = 0i32;

    while (index as i64 as usize) < ctx.counter_surge_events.len() {
        let at = index as i64 as usize;
        let missing = Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: ctx.counter_surge_events.len() as i64 };

        if ctx.counter_surge_events.get(at).ok_or(missing.clone())?.state == 1 {
            ctx.counter_surge_events.remove(at);
            index = index.wrapping_sub(1);
            index = index.wrapping_add(1);

            continue;
        }

        let before = ctx.counter_surge_events.get(at).ok_or(missing.clone())?.frame;

        if before == get_setting(&ctx.settings, b"battle_demon_summon_se_frame", 0)? {
            play_sound(sound_manager(ctx)?, 0x9f, None);
        }

        let event = ctx.counter_surge_events.get_mut(at).ok_or(missing.clone())?;
        let faction = event.faction;
        let frame = event.frame.wrapping_add(1);

        event.frame = frame;

        if frame == get_setting(&ctx.settings, b"battle_demon_summon_frame", 0x1e)? {
            let slot = ctx.counter_surge_events.get(at).ok_or(missing.clone())?.slot;

            roll_procs(ctx, faction, slot, 0)?;

            let proc_flags = [
                (ctx.i32_at(AppContext::PROC_ROLLS)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x4)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x8)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0xc)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x10)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x18)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x20)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x1c)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x14)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x24)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x28)? != 0) as u8,
                (ctx.i32_at(AppContext::PROC_ROLLS + 0x2c)? != 0) as u8,
            ];

            ctx.surge_events.push(SurgeEvent::default());

            let surge = ctx.surge_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            surge.faction = faction;
            surge.slot = slot;
            surge.frame = 0;

            let x = get_pos_x(ctx, faction, slot)?;
            let anchor = ctx.counter_surge_events.get(at).ok_or(missing.clone())?.anchor;
            let reach = call_rng(ctx, abs_i32(ctx.counter_surge_events.get(at).ok_or(missing.clone())?.span));
            let event = ctx.counter_surge_events.get(at).ok_or(missing.clone())?;
            let offset = (if event.span > 0 { reach } else { reach.wrapping_neg() }).wrapping_add(anchor);
            let level = event.level;
            let surge = ctx.surge_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            surge.x = (if faction == 1 { offset } else { offset.wrapping_neg() }).wrapping_add(x);
            surge.level = level;
            surge.attack = 0;
            surge.proc_flags = proc_flags;

            let metal_killer_pct = get_metal_killer_pct(ctx, faction, slot)?;
            let mini = ctx.counter_surge_events.get(at).ok_or(missing.clone())?.mini;
            let surge = ctx.surge_events.last_mut().ok_or(Fault::IndexOutOfRange { site: SITE, index: 0, limit: 0 })?;

            surge.metal_killer_pct = metal_killer_pct;
            surge.mini = mini;
            surge.kind = 1;
            index = index.wrapping_add(1);

            continue;
        }

        let event = ctx.counter_surge_events.get(at).ok_or(missing.clone())?;

        if event.frame >= get_setting(&ctx.settings, b"battle_demon_summon_frame", 0x1e)? {
            let frame = ctx.counter_surge_events.get(at).ok_or(missing.clone())?.frame;
            let anim = ctx.counter_surge_anims.get(faction as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: faction as i64, limit: 2 })?;

            if frame >= get_anim_len(anim)? {
                ctx.counter_surge_events.remove(at);
                index = index.wrapping_sub(1);
            }
        }

        index = index.wrapping_add(1);
    }

    Ok(())
}
