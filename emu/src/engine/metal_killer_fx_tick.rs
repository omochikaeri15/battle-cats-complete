use crate::Fault;

use super::{get_anim_len, AppContext};

const SITE: &str = "metal_killer_fx_tick";

pub fn metal_killer_fx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let count = ctx.metal_killer_fx.len() as i32;

    if count <= 0 {
        return Ok(());
    }

    let mut index = count as usize;

    loop {
        let at = index - 1;
        let limit = ctx.metal_killer_fx.len() as i64;
        let frame = ctx.metal_killer_fx.get(at).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit })?.frame.wrapping_add(1);

        ctx.metal_killer_fx.get_mut(at).ok_or(Fault::IndexOutOfRange { site: SITE, index: at as i64, limit })?.frame = frame;

        if frame >= get_anim_len(&ctx.metal_killer_fx_anim)? {
            ctx.metal_killer_fx.remove(at);
        }

        if index <= 1 {
            return Ok(());
        }

        index -= 1;
    }
}
