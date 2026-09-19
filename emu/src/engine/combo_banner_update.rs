use crate::Fault;

use super::{
    AppContext, combo_banner_pending, combo_banner_skip_all, get_drawable_width,
    get_nyancombo_text, get_text_texture, hit_test_rect, set_combo_banner_pending,
    text_texture_cache, touch_released,
};

pub fn combo_banner_update(ctx: &mut AppContext) -> Result<(), Fault> {
    if !ctx
        .combo_store
        .records
        .iter()
        .any(|record| combo_banner_pending(record) != 0)
    {
        ctx.set_block_at::<16>(AppContext::COMBO_BANNER_STATE, [0; 16])?;
        ctx.set_block_at::<16>(AppContext::COMBO_BANNER_UNITS, [0xff; 16])?;
        ctx.set_i32_at(AppContext::COMBO_BANNER_UNITS.wrapping_add(0x10), -1)?;

        ctx.combo_banner_texts[0] = None;
        ctx.combo_banner_texts[1] = None;
        ctx.combo_banner_texts[2] = None;

        return Ok(());
    }

    let step_frames = ctx.i32_at(AppContext::COMBO_BANNER_STEP_FRAMES)?;
    let tick = ctx.i32_at(AppContext::COMBO_BANNER_STATE)?.wrapping_add(1);

    ctx.set_i32_at(AppContext::COMBO_BANNER_STATE, tick)?;
    ctx.set_i32_at(
        AppContext::COMBO_BANNER_TICKS,
        ctx.i32_at(AppContext::COMBO_BANNER_TICKS)?.wrapping_add(1),
    )?;

    if tick < step_frames {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::COMBO_BANNER_STATE, step_frames)?;
    ctx.set_i32_at(
        AppContext::COMBO_BANNER_PHASE,
        ctx.i32_at(AppContext::COMBO_BANNER_PHASE)?.wrapping_add(1),
    )?;

    let text_step = ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)?;
    let sub = ctx.i32_at(AppContext::COMBO_BANNER_SUB)?.wrapping_add(1);

    ctx.set_i32_at(AppContext::COMBO_BANNER_SUB, sub)?;

    if sub >= text_step {
        ctx.set_i32_at(AppContext::COMBO_BANNER_SUB, text_step)?;

        let x = ctx
            .i32_at(AppContext::COMBO_BANNER_X)?
            .wrapping_add(ctx.i32_at(AppContext::COMBO_BANNER_SPEED)?);

        ctx.set_i32_at(AppContext::COMBO_BANNER_X, x)?;

        if x >= get_drawable_width(ctx)? {
            let width = get_drawable_width(ctx)?;

            ctx.set_i32_at(AppContext::COMBO_BANNER_X, width)?;
        }
    }

    let phase = ctx.i32_at(AppContext::COMBO_BANNER_PHASE)?;

    let build = if phase == 1 {
        ctx.set_block_at::<16>(AppContext::COMBO_BANNER_UNITS, [0xff; 16])?;
        ctx.set_i32_at(AppContext::COMBO_BANNER_UNITS.wrapping_add(0x10), -1)?;

        match ctx
            .combo_store
            .records
            .iter()
            .find(|record| combo_banner_pending(record) != 0)
            .map(|record| record.unit)
        {
            Some(units) => {
                for (index, unit) in units.iter().enumerate() {
                    ctx.set_i32_at(
                        AppContext::COMBO_BANNER_UNITS.wrapping_add(index * 4),
                        *unit,
                    )?;
                }

                ctx.i32_at(AppContext::COMBO_BANNER_PHASE)?
                    == ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)?
            }
            None => ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)? == 1,
        }
    } else {
        ctx.i32_at(AppContext::COMBO_BANNER_TEXT_STEP)? == phase
    };

    if build {
        ctx.combo_banner_texts[0] = None;
        ctx.combo_banner_texts[1] = None;
        ctx.combo_banner_texts[2] = None;

        if let Some(record) = ctx
            .combo_store
            .records
            .iter()
            .find(|record| combo_banner_pending(record) != 0)
            .cloned()
        {
            let font = ctx.default_font.clone();
            let name = get_nyancombo_text(ctx, &record, 0)?;

            ctx.combo_banner_texts[0] = Some(get_text_texture(
                text_texture_cache(ctx)?,
                &name,
                &font,
                0x28,
                1,
                0,
            ));

            let effect = get_nyancombo_text(ctx, &record, 1)?;

            ctx.combo_banner_texts[1] = Some(get_text_texture(
                text_texture_cache(ctx)?,
                &effect,
                &font,
                0x1a,
                1,
                0,
            ));

            if record.charagroup_id != -1 {
                let group = get_nyancombo_text(ctx, &record, 2)?;

                ctx.combo_banner_texts[2] = Some(get_text_texture(
                    text_texture_cache(ctx)?,
                    &group,
                    &font,
                    0x1a,
                    1,
                    0,
                ));
            }
        }
    }

    if ctx.i32_at(AppContext::COMBO_BANNER_PHASE)?
        >= ctx.i32_at(AppContext::COMBO_BANNER_LIFETIME)?
    {
        ctx.set_block_at::<8>(AppContext::COMBO_BANNER_PHASE, [0; 8])?;
        ctx.set_i32_at(AppContext::COMBO_BANNER_SUB, 0)?;
        ctx.set_block_at::<16>(AppContext::COMBO_BANNER_UNITS, [0xff; 16])?;
        ctx.set_i32_at(AppContext::COMBO_BANNER_UNITS.wrapping_add(0x10), -1)?;

        if let Some(record) = ctx
            .combo_store
            .records
            .iter_mut()
            .find(|record| combo_banner_pending(record) != 0)
        {
            set_combo_banner_pending(record, 0);
        }
    }

    let shift = ctx
        .combo_store
        .records
        .iter()
        .find(|record| combo_banner_pending(record) != 0)
        .map_or(0, |record| if record.charagroup_id == -1 { 0 } else { 0xf });

    if touch_released(ctx)? == 0 {
        return Ok(());
    }

    let x = ctx.i32_at(AppContext::COMBO_SKIP_RECT)?;
    let y = ctx
        .i32_at(AppContext::COMBO_SKIP_RECT.wrapping_add(4))?
        .wrapping_add(shift);
    let w = ctx.i32_at(AppContext::COMBO_SKIP_RECT.wrapping_add(8))?;
    let h = ctx.i32_at(AppContext::COMBO_SKIP_RECT.wrapping_add(0xc))?;

    if hit_test_rect(ctx, x, y, w, h)? {
        combo_banner_skip_all(ctx)?;
    }

    Ok(())
}
