use crate::Fault;

use super::{get_anim_len, play_sound_in_battle, AppContext};

const SITE: &str = "toxic_fx_tick";

pub fn toxic_fx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let count = ctx.toxic_fx.len() as i32;

    if count <= 0 {
        return Ok(());
    }

    let mut index = count as usize;

    loop {
        let at = index - 1;
        let limit = ctx.toxic_fx.len() as i64;
        let mut frame = ctx.toxic_fx.get(at).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit })?.frame;

        if frame == 0 {
            play_sound_in_battle(ctx, 0x6e)?;
            frame = ctx.toxic_fx.get(at).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit })?.frame;
        }

        frame = frame.wrapping_add(1);
        ctx.toxic_fx.get_mut(at).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit })?.frame = frame;

        if frame >= get_anim_len(&ctx.toxic_fx_anim)? {
            ctx.toxic_fx.remove(at);
        }

        if index <= 1 {
            return Ok(());
        }

        index -= 1;
    }
}
