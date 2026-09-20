use crate::{Fault, ops};

use super::{
    AppContext, CANNON_SHOT_SPACING, WaveRecord, draw_context, draw_model, get_anim_len,
    get_drawable_width, maanim_execute,
};

pub fn wave_draw(ctx: &mut AppContext, depth: i32, slot: i32, side: i32) -> Result<(), Fault> {
    let y = depth.wrapping_add(0x1cc);

    for record in 0..0xc8usize {
        let base = AppContext::WAVE_RECORDS + record * AppContext::WAVE_RECORD_STRIDE;

        for sprite in 0..6usize {
            let cell =
                AppContext::WAVE_SPRITES + record * AppContext::WAVE_RECORD_STRIDE + sprite * 8;

            if ctx.i32_at(cell)? == 0 {
                continue;
            }

            if ctx.i32_at(base + WaveRecord::OWNER_SLOT)? != slot {
                continue;
            }

            if ctx.i32_at(base + WaveRecord::IN_USE)? != side {
                continue;
            }

            let pos = ctx
                .i32_at(cell + 4)?
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
            let shift = if ctx.i32_at(base + WaveRecord::KIND)? != 2 {
                0
            } else {
                CANNON_SHOT_SPACING.wrapping_mul(5).wrapping_add(-0x21c)
            };
            let pos = shift.wrapping_add(pos);
            let mini = ctx.u8_at(base + WaveRecord::MINI)?;
            let counter = ctx.i32_at(cell)?;
            let half = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5;
            let x = ops::cvttsd2si(half + ops::div_10(pos) as f64);
            let (model, anim) = match (side.wrapping_sub(1), mini) {
                (0, 0) => (&mut ctx.wave_attack_model, &ctx.wave_anim),
                (0, _) => (&mut ctx.smallwave_attack_model, &ctx.mini_wave_anim),
                (1, 0) => (&mut ctx.wave_attack_e_model, &ctx.wave_attack_e_anim),
                (1, _) => (
                    &mut ctx.smallwave_attack_e_model,
                    &ctx.smallwave_attack_e_anim,
                ),
                (index, _) => {
                    return Err(Fault::index_out_of_range(index as i64, 2));
                }
            };
            let frame = get_anim_len(anim)?.wrapping_add(!counter);

            maanim_execute(model, Some(anim), frame, 0)?;
            draw_model(draw_context(&mut ctx.draw)?, model, x, y);
        }
    }

    Ok(())
}
