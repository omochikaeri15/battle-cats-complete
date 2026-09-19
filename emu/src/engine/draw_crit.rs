use crate::{Fault, operation};

use super::{AppContext, VfxSlot, draw_context, draw_model, get_drawable_width, maanim_execute};

pub fn draw_crit(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0xc8usize {
        let slot =
            AppContext::CRIT_VFX.wrapping_add(record.wrapping_mul(AppContext::CRIT_VFX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(VfxSlot::ACTIVE))? == 0 {
            continue;
        }

        let pos_x = ctx
            .i32_at(slot.wrapping_add(VfxSlot::POS_X))?
            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, operation::div_10(pos_x))?;

        let frame = ctx.i32_at(slot.wrapping_add(VfxSlot::FRAME))?;

        maanim_execute(&mut ctx.crit_vfx_model, Some(&ctx.crit_vfx_anim), frame, 0)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)?;
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
            .wrapping_add(origin)
            .wrapping_add(0x44);
        let y =
            operation::div_10(ctx.i32_at(slot.wrapping_add(VfxSlot::POS_Y))?).wrapping_add(0x4d);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.crit_vfx_model, x, y);
    }

    Ok(())
}
