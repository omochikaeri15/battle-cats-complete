use crate::{Fault, ops};

use super::{AppContext, VfxSlot, draw_context, draw_model, get_drawable_width, maanim_execute};

pub fn barrier_vfx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot = AppContext::BARRIER_VFX
            .wrapping_add(record.wrapping_mul(AppContext::BARRIER_VFX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(VfxSlot::ACTIVE))? == 0 {
            continue;
        }

        let broken = ctx.u8_at(slot.wrapping_add(VfxSlot::BROKEN))?;
        let frame = ctx.i32_at(slot.wrapping_add(VfxSlot::FRAME))?;
        let anim = if broken == 0 {
            &ctx.barrier_anims[1]
        } else {
            &ctx.barrier_anims[2]
        };

        maanim_execute(&mut ctx.barrier_model, Some(anim), frame, 0)?;

        let pos_x = ops::div_10(
            ctx.i32_at(slot.wrapping_add(VfxSlot::POS_X))?
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pos_x);
        let y = ops::div_10(ctx.i32_at(slot.wrapping_add(VfxSlot::POS_Y))?);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.barrier_model, x, y);
    }

    Ok(())
}
