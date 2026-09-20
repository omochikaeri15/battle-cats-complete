use std::{cell::Cell, rc::Rc};

use crate::{Fault, operation};

use super::{
    AppContext, bg_param_resolve_int, draw_context, draw_model, fill_polygon_colored, fill_rect,
    get_background_id, get_bg_gradient_bottom, get_bg_gradient_top, get_design_height2,
    get_drawable_width, has_bg_gradient, maanim_execute, mamodel_get_angle_unit,
    mamodel_get_opacity_unit, mamodel_get_part, mamodel_get_scale_unit, mamodel_set_sheet,
    mamodel_set_sheet_table, set_part_angle, set_part_opacity, set_part_scale, set_tint,
};

pub fn draw_background_overlay(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_background_id(ctx)? == 0xd
        || get_background_id(ctx)? == 0xf
        || get_background_id(ctx)? == 0x13
        || get_background_id(ctx)? == 0x47
        || get_background_id(ctx)? == 0x48
        || get_background_id(ctx)? == 0x49
        || get_background_id(ctx)? == 0xa7
        || get_background_id(ctx)? == 0xa8
    {
        ctx.set_block_at::<8>(AppContext::BG_TINT_XS, [0; 8])?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_XS + 0xc, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_XS + 8, width)?;

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        ctx.set_i32_at(AppContext::BG_TINT_YS + 0xc, top)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS, top)?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_YS + 8, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS + 4, width)?;

        let pair = if get_background_id(ctx)? == 0xd {
            Some((0x33ffffffu32, 0x3300e2ffu32))
        } else if get_background_id(ctx)? == 0xf {
            Some((0x3342bbff, 0x330049ad))
        } else if get_background_id(ctx)? == 0x13 {
            Some((0x33f0a936, 0x33a02120))
        } else if get_background_id(ctx)? == 0x47 {
            Some((0x33eba03c, 0x33912d05))
        } else if get_background_id(ctx)? == 0x48 {
            Some((0x33000000, 0x3300237d))
        } else if get_background_id(ctx)? == 0x49 {
            Some((0x333c8705, 0x330f1e78))
        } else if get_background_id(ctx)? == 0xa7 {
            Some((0x4c896858, 0x4caf6868))
        } else if get_background_id(ctx)? == 0xa8 {
            Some((0x4c444b4d, 0x4c314961))
        } else {
            None
        };

        if let Some((upper, lower)) = pair {
            ctx.set_i32_at(AppContext::BG_TINT_COLORS, upper as i32)?;
            ctx.set_i32_at(AppContext::BG_TINT_COLORS + 4, lower as i32)?;
            ctx.set_i32_at(AppContext::BG_TINT_COLORS + 8, lower as i32)?;
            ctx.set_i32_at(AppContext::BG_TINT_COLORS + 0xc, upper as i32)?;
        }

        let mut xs = [0i32; 4];
        let mut ys = [0i32; 4];
        let mut colors = [0u32; 4];

        for corner in 0..4usize {
            xs[corner] = ctx.i32_at(AppContext::BG_TINT_XS + corner * 4)?;
            ys[corner] = ctx.i32_at(AppContext::BG_TINT_YS + corner * 4)?;
            colors[corner] = ctx.i32_at(AppContext::BG_TINT_COLORS + corner * 4)? as u32;
        }

        fill_polygon_colored(draw_context(&mut ctx.draw)?, &xs, &ys, &colors, 4);
    } else if get_background_id(ctx)? == 0x2e || get_background_id(ctx)? == 0x2f {
        ctx.set_block_at::<8>(AppContext::BG_TINT_XS, [0; 8])?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_XS + 0xc, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_XS + 8, width)?;

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        ctx.set_i32_at(AppContext::BG_TINT_YS + 0xc, top)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS, top)?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_YS + 8, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS + 4, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS, 0x4cffffffu32 as i32)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 4, 0x33ffffffu32 as i32)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 8, 0x33ffffffu32 as i32)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 0xc, 0x4cffffffu32 as i32)?;

        let mut xs = [0i32; 4];
        let mut ys = [0i32; 4];
        let mut colors = [0u32; 4];

        for corner in 0..4usize {
            xs[corner] = ctx.i32_at(AppContext::BG_TINT_XS + corner * 4)?;
            ys[corner] = ctx.i32_at(AppContext::BG_TINT_YS + corner * 4)?;
            colors[corner] = ctx.i32_at(AppContext::BG_TINT_COLORS + corner * 4)? as u32;
        }

        fill_polygon_colored(draw_context(&mut ctx.draw)?, &xs, &ys, &colors, 4);
    } else if get_background_id(ctx)? == 0x9c {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0x4c);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    }

    if !ctx.bg_effects.instances.is_empty() {
        let mut index = 0usize;

        loop {
            let instance = &ctx.bg_effects.instances[index];

            if instance.z == 0 && instance.wait <= 0 {
                let key = instance.model;
                let name = ctx
                    .bg_effects
                    .model_names
                    .get(&key)
                    .ok_or(Fault::key_not_found(key as i64))?
                    .clone();
                let mut model =
                    std::mem::take(ctx.bg_models.get_mut(&name).ok_or(Fault::key_not_found(key as i64))?);
                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;

                if instance.model < 0 {
                    let key = instance.model;
                    let image = ctx
                        .bg_effects
                        .image_names
                        .get(&key)
                        .ok_or(Fault::key_not_found(key as i64))?;
                    let sheet = ctx
                        .bg_effect_sheets
                        .get(image)
                        .ok_or(Fault::key_not_found(key as i64))?
                        .clone();

                    mamodel_set_sheet(&mut model, sheet);
                } else {
                    mamodel_set_sheet_table(
                        &mut model,
                        &Rc::from([Cell::new(ctx.bg_sheet.clone())]),
                    );
                }

                let part = mamodel_get_part(&model, 0).ok_or(Fault::null_pointer())?;
                let unit = mamodel_get_scale_unit(&model);
                let scale = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?
                    .scale;
                let unit_y = mamodel_get_scale_unit(&model);
                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;
                let width = operation::cvttss2si(scale * unit as f32);
                let height = operation::cvttss2si(unit_y as f32 * instance.scale);

                set_part_scale(&mut model.parts[part], width, height);

                let part = mamodel_get_part(&model, 0).ok_or(Fault::null_pointer())?;
                let unit = mamodel_get_angle_unit(&model);
                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;

                set_part_angle(
                    &mut model.parts[part],
                    operation::cvttss2si(unit as f32 * instance.angle / 360.0),
                );

                let part = mamodel_get_part(&model, 0).ok_or(Fault::null_pointer())?;
                let unit = mamodel_get_opacity_unit(&model);
                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;

                set_part_opacity(
                    &mut model.parts[part],
                    operation::cvttss2si(unit as f32 * instance.alpha / 255.0),
                );

                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;
                let key = instance.model;
                let anim_name = ctx
                    .bg_effects
                    .model_anims
                    .get(&key)
                    .ok_or(Fault::key_not_found(key as i64))?;
                let anim = ctx.bg_anim_cache.get(anim_name).ok_or(Fault::key_not_found(key as i64))?;
                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;

                maanim_execute(&mut model, Some(anim), instance.frame, 0)?;

                let instance = ctx
                    .bg_effects
                    .instances
                    .get(index)
                    .ok_or(Fault::out_of_range())?;
                let def_index = instance.def_index;
                let def = ctx.bg_effects.defs.get(def_index as i64 as usize).ok_or(
                    Fault::index_out_of_range(def_index as i64, ctx.bg_effects.defs.len() as i64),
                )?;
                let first = def.equally_spaced.pos1;
                let last = def.equally_spaced.pos2;

                if first <= last {
                    let value = def.equally_spaced.value;
                    let base = def.equally_spaced.base;
                    let end = last.wrapping_add(1);
                    let mut step = first;

                    loop {
                        let origin = ctx
                            .bg_effects
                            .instances
                            .get(index)
                            .ok_or(Fault::out_of_range())?
                            .x;
                        let spacing = bg_param_resolve_int(ctx, -1, value, base)?;
                        let instance = ctx
                            .bg_effects
                            .instances
                            .get(index)
                            .ok_or(Fault::out_of_range())?;
                        let x = operation::cvttss2si(origin + spacing.wrapping_mul(step) as f32);
                        let y = operation::cvttss2si(instance.y);

                        draw_model(draw_context(&mut ctx.draw)?, &model, x, y);
                        step = step.wrapping_add(1);

                        if step == end {
                            break;
                        }
                    }
                }

                *ctx.bg_models.get_mut(&name).ok_or(Fault::key_not_found(key as i64))? = model;
            }

            index += 1;

            if index >= ctx.bg_effects.instances.len() {
                break;
            }
        }
    }

    if has_bg_gradient(ctx, AppContext::BG_SETUP)? {
        ctx.set_block_at::<8>(AppContext::BG_TINT_XS, [0; 8])?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_XS + 0xc, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_XS + 8, width)?;

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        ctx.set_i32_at(AppContext::BG_TINT_YS + 0xc, top)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS, top)?;

        let width = get_drawable_width(ctx)?;

        ctx.set_i32_at(AppContext::BG_TINT_YS + 8, width)?;
        ctx.set_i32_at(AppContext::BG_TINT_YS + 4, width)?;

        let upper = get_bg_gradient_top(ctx, AppContext::BG_SETUP)?;

        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 0xc, upper)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS, upper)?;

        let lower = get_bg_gradient_bottom(ctx, AppContext::BG_SETUP)?;

        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 8, lower)?;
        ctx.set_i32_at(AppContext::BG_TINT_COLORS + 4, lower)?;

        let mut xs = [0i32; 4];
        let mut ys = [0i32; 4];
        let mut colors = [0u32; 4];

        for corner in 0..4usize {
            xs[corner] = ctx.i32_at(AppContext::BG_TINT_XS + corner * 4)?;
            ys[corner] = ctx.i32_at(AppContext::BG_TINT_YS + corner * 4)?;
            colors[corner] = ctx.i32_at(AppContext::BG_TINT_COLORS + corner * 4)? as u32;
        }

        fill_polygon_colored(draw_context(&mut ctx.draw)?, &xs, &ys, &colors, 4);
    }

    Ok(())
}
