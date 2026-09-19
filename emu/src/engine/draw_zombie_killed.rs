use crate::{operation, Fault};

use super::{draw_context, draw_model, get_drawable_width, maanim_execute, AppContext, FxSlot};

pub fn draw_zombie_killed(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot = AppContext::ZKILL_FX.wrapping_add(record.wrapping_mul(AppContext::ZKILL_FX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(FxSlot::ACTIVE))? == 0 {
            continue;
        }

        let frame = ctx.i32_at(slot.wrapping_add(FxSlot::FRAME))?;

        maanim_execute(&mut ctx.skill_zombie_strong_model, Some(&ctx.zkill_fx_anim), frame, 0)?;

        let pos_x = operation::div_10(ctx.i32_at(slot.wrapping_add(FxSlot::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pos_x);
        let y = operation::div_10(ctx.i32_at(slot.wrapping_add(FxSlot::POS_Y))?).wrapping_add(-0x28);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.skill_zombie_strong_model, x, y);
    }

    Ok(())
}
