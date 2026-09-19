use crate::{operation, Fault};

use super::{draw_context, draw_model, get_drawable_width, maanim_execute, AppContext, FxSlot};

pub fn shield_fx_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    for record in 0..0x1eusize {
        let slot = AppContext::SHIELD_FX.wrapping_add(record.wrapping_mul(AppContext::SHIELD_FX_STRIDE));

        if ctx.u8_at(slot.wrapping_add(FxSlot::ACTIVE))? == 0 {
            continue;
        }

        let broken = ctx.u8_at(slot.wrapping_add(FxSlot::BROKEN))?;
        let frame = ctx.i32_at(slot.wrapping_add(FxSlot::FRAME))?;
        let anim = if broken == 0 { &ctx.shield_anims[3] } else { &ctx.shield_anims[4] };

        maanim_execute(&mut ctx.demonshield_model, Some(anim), frame, 0)?;

        let pos_x = operation::div_10(ctx.i32_at(slot.wrapping_add(FxSlot::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pos_x);
        let y = operation::div_10(ctx.i32_at(slot.wrapping_add(FxSlot::POS_Y))?);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.demonshield_model, x, y);
    }

    Ok(())
}
