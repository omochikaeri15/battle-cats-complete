use crate::Fault;

use super::{AppContext, get_anim_len};

pub fn metal_killer_vfx_tick(ctx: &mut AppContext) -> Result<(), Fault> {
    let count = ctx.metal_killer_vfx.len() as i32;

    if count <= 0 {
        return Ok(());
    }

    let mut index = count as usize;

    loop {
        let at = index - 1;
        let limit = ctx.metal_killer_vfx.len() as i64;
        let frame = ctx
            .metal_killer_vfx
            .get(at)
            .ok_or(Fault::index_out_of_range(at as i64, limit))?
            .frame
            .wrapping_add(1);

        ctx.metal_killer_vfx
            .get_mut(at)
            .ok_or(Fault::index_out_of_range(at as i64, limit))?
            .frame = frame;

        if frame >= get_anim_len(&ctx.metal_killer_vfx_anim)? {
            ctx.metal_killer_vfx.remove(at);
        }

        if index <= 1 {
            return Ok(());
        }

        index -= 1;
    }
}
