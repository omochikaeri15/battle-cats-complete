use std::{cell::Cell, rc::Rc};

use crate::{Fault, ops};

use super::{
    AppContext, Surface, bg_param_resolve_int, cos_deg, draw_context, draw_cut_rotated,
    draw_cut_scaled, draw_cut_spun, draw_image_rotated, draw_model, draw_surface_scaled,
    fill_polygon, get_background_id, get_drawable_width, imgcut_get_sprite_cut, maanim_execute,
    mamodel_get_angle_unit, mamodel_get_opacity_unit, mamodel_get_part, mamodel_get_scale_unit,
    mamodel_set_sheet, mamodel_set_sheet_table, set_alpha, set_part_angle, set_part_opacity,
    set_part_scale, set_tint, sin_deg,
};

pub fn draw_foreground_effects(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_background_id(ctx)? == 0x29
        || get_background_id(ctx)? == 0x4b
        || get_background_id(ctx)? == 0x3f0
    {
        for sprite in 0..0x1eusize {
            let base = AppContext::BG_SPRITES + sprite * 0x40;
            let fade = ctx.i32_at(base + 0x28)?;

            if fade <= 0x1d {
                set_alpha(
                    draw_context(&mut ctx.draw)?,
                    ops::div_30((fade << 8).wrapping_sub(fade)),
                );
            }

            let sheet = ctx.bg_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let left = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?)
                .wrapping_add(ops::div_100(ctx.i32_at(base + 0x20)?));
            let width = get_drawable_width(ctx)?;
            let phase = ctx.i32_at(base + 0x28)?;
            let odd =
                phase.wrapping_sub(phase.wrapping_add((phase as u32 >> 31) as i32) & 0x7ffffffe);
            let x = left
                .wrapping_sub(width)
                .wrapping_add(odd.wrapping_mul(2))
                .wrapping_add(0x3c4);
            let y = ops::div_100(ctx.i32_at(base + 0x24)?);
            let cut = ctx.i32_at(base + 0x2c)?;
            let span = ops::div_100(
                ctx.i32_at(base + 0x30)?
                    .wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[2]),
            );
            let cut = ctx.i32_at(base + 0x2c)?;
            let height = ops::div_100(
                ctx.i32_at(base + 0x30)?
                    .wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[3]),
            );
            let cut = ctx.i32_at(base + 0x2c)?;
            let angle = ctx.i32_at(base + 0x38)? as f32;

            draw_cut_rotated(
                draw_context(&mut ctx.draw)?,
                sheet,
                x,
                y,
                span,
                height,
                angle,
                5,
                0,
                0,
                5,
                cut,
            );
            set_alpha(draw_context(&mut ctx.draw)?, 0xff);
        }
    }

    if get_background_id(ctx)? == 3
        || get_background_id(ctx)? == 0x1b
        || get_background_id(ctx)? == 0xc5
    {
        let left = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?).wrapping_add(0x3c0);

        for flake in 0..100usize {
            let drifter = AppContext::BG_DRIFTERS + flake * 0x10;
            let polygon = AppContext::BG_SPRITES + flake * 0x40;

            for corner in 0..8usize {
                let turn = (corner * 0x2d) as i32 as f32;
                let across = ops::div_100(ctx.i32_at(drifter)?)
                    .wrapping_add(left)
                    .wrapping_sub(get_drawable_width(ctx)?) as f32;
                let x = ops::cvttss2si(cos_deg(turn) * 12.0 + across);

                ctx.set_i32_at(polygon + corner * 4, x)?;

                let down = ops::div_100(ctx.i32_at(drifter + 4)?) as f32;
                let y = ops::cvttss2si(sin_deg(turn) * 12.0 + down);

                ctx.set_i32_at(polygon + 0x20 + corner * 4, y)?;
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0x7f);

            let mut xs = [0i32; 8];
            let mut ys = [0i32; 8];

            for corner in 0..8usize {
                xs[corner] = ctx.i32_at(polygon + corner * 4)?;
                ys[corner] = ctx.i32_at(polygon + 0x20 + corner * 4)?;
            }

            fill_polygon(draw_context(&mut ctx.draw)?, &xs, &ys, 8);
        }
    }

    if get_background_id(ctx)? == 0x21 || get_background_id(ctx)? == 0x3a {
        for star in 0..0x3cusize {
            let base = AppContext::BG_STARS + star * 0x10;
            let alpha = ctx.i32_at(base + 8)?;

            set_alpha(draw_context(&mut ctx.draw)?, alpha);

            let x = ctx.i32_at(base)?;
            let y = ctx.i32_at(base + 4)?;

            draw_cut_spun(
                draw_context(&mut ctx.draw)?,
                ctx.effect_a_sheet
                    .as_deref()
                    .ok_or(Fault::null_pointer())?,
                x,
                y,
                5,
                0,
                60.0,
                0,
                5,
                0x1d,
            );
        }

        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
    }

    if get_background_id(ctx)? == 0xd
        || get_background_id(ctx)? == 0xf
        || get_background_id(ctx)? == 0x48
    {
        for bubble in 0..100usize {
            let base = AppContext::BG_PARTICLES + bubble * 0x14;
            let sheet = ctx.bubble_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let across = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?)
                .wrapping_add(ctx.i32_at(base)?)
                .wrapping_sub(get_drawable_width(ctx)?)
                .wrapping_add(0x3c0) as f32;
            let sway = ctx.i32_at(base + 8)?.wrapping_mul(0x12) as f32;
            let x = ops::cvttss2si(cos_deg(sway) * 10.0 + across);
            let y = ctx.i32_at(base + 4)?;

            draw_surface_scaled(
                draw_context(&mut ctx.draw)?,
                Surface::Sheet(sheet),
                x,
                y,
                0x13,
                0x13,
            );
        }
    } else if get_background_id(ctx)? == 0x51
        || get_background_id(ctx)? == 0x65
        || get_background_id(ctx)? == 0x7b
        || get_background_id(ctx)? == 0x92
        || get_background_id(ctx)? == 0xa9
    {
        for petal in 0..0x32usize {
            let base = AppContext::BG_PARTICLES + petal * 0x14;
            let sheet = ctx.bg_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let cut = ctx.i32_at(base + 0x10)?.wrapping_add(0x14);
            let wide = ctx
                .i32_at(base + 0xc)?
                .wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[2]);
            let width = ops::div_100(wide);
            let cut = ctx.i32_at(base + 0x10)?.wrapping_add(0x14);
            let tall = ctx
                .i32_at(base + 0xc)?
                .wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[3]);
            let height = ops::div_100(tall);
            let across = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?)
                .wrapping_add(ctx.i32_at(base)?)
                .wrapping_sub(get_drawable_width(ctx)?)
                .wrapping_add(0x3c0) as f32;
            let sway = ctx.i32_at(base + 8)?.wrapping_mul(3) as f32;
            let x = ops::cvttss2si(
                cos_deg(sway) * 10.0 + across - ops::div_200(wide) as f32,
            );
            let y = ops::div_neg_200(tall).wrapping_add(ctx.i32_at(base + 4)?);
            let cut = ctx.i32_at(base + 0x10)?.wrapping_add(0x14);

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                sheet,
                x,
                y,
                width,
                height,
                cut,
            );
        }
    }

    if get_background_id(ctx)? == 0x28 {
        for pair in 0..100usize {
            let base = AppContext::BG_SPRITES + pair * 0x40;

            for half in [0usize, 0x20] {
                let sheet = ctx.bubble_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                let across = ops::div_100(ctx.i32_at(base + half)?)
                    .wrapping_add(ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?))
                    .wrapping_sub(get_drawable_width(ctx)?)
                    .wrapping_add(0x3c0) as f32;
                let size = (ctx.i32_at(base + half + 0xc)? << 2) as f32;
                let sway = ops::div_100(
                    ctx.i32_at(base + half + 0x10)?
                        .wrapping_mul(ctx.i32_at(base + half + 8)?),
                ) as f32;
                let x = ops::cvttss2si(cos_deg(sway) * size / 100.0 + across);
                let y = ops::div_100(ctx.i32_at(base + half + 4)?);
                let size = ops::div_5(ctx.i32_at(base + half + 0xc)?);

                draw_surface_scaled(
                    draw_context(&mut ctx.draw)?,
                    Surface::Sheet(sheet),
                    x,
                    y,
                    size,
                    size,
                );
            }
        }
    }

    if get_background_id(ctx)? == 0x2e || get_background_id(ctx)? == 0x2f {
        for pair in 0..100usize {
            let base = AppContext::BG_SPRITES + pair * 0x40;

            for half in [0usize, 0x20] {
                let sheet = ctx.bubble_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
                let across = ops::div_100(ctx.i32_at(base + half)?)
                    .wrapping_add(ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?))
                    .wrapping_sub(get_drawable_width(ctx)?)
                    .wrapping_add(0x3c0) as f32;
                let size = (ctx.i32_at(base + half + 0xc)? << 2) as f32;
                let sway = ops::div_100(
                    ctx.i32_at(base + half + 0x10)?
                        .wrapping_mul(ctx.i32_at(base + half + 8)?),
                ) as f32;
                let x = ops::cvttss2si(cos_deg(sway) * size / 100.0 + across);
                let y = ops::div_100(ctx.i32_at(base + half + 4)?);
                let width = ops::div_10(ctx.i32_at(base + half + 0xc)?);
                let height = ops::div_5(ctx.i32_at(base + half + 0xc)?);

                draw_image_rotated(
                    draw_context(&mut ctx.draw)?,
                    sheet,
                    x,
                    y,
                    width,
                    height,
                    315.0,
                    0,
                    width,
                    height,
                    0,
                );
            }
        }
    }

    if ctx.bg_effects.instances.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        let instance = &ctx.bg_effects.instances[index];

        if instance.z == 1 && instance.wait <= 0 {
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
                mamodel_set_sheet_table(&mut model, &Rc::from([Cell::new(ctx.bg_sheet.clone())]));
            }

            let part = mamodel_get_part(&model, 0).ok_or(Fault::null_pointer())?;
            let unit = mamodel_get_scale_unit(&model);
            let instance = ctx
                .bg_effects
                .instances
                .get(index)
                .ok_or(Fault::out_of_range())?;
            let scale = instance.scale;
            let scale_x = instance.scale_x;
            let unit_y = mamodel_get_scale_unit(&model);
            let instance = ctx
                .bg_effects
                .instances
                .get(index)
                .ok_or(Fault::out_of_range())?;
            let width = ops::cvttss2si(scale * unit as f32 * scale_x);
            let height = ops::cvttss2si(unit_y as f32 * instance.scale * instance.scale_y);

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
                ops::cvttss2si(unit as f32 * instance.angle / 360.0),
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
                ops::cvttss2si(unit as f32 * instance.alpha / 255.0),
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
                    let camera = ctx.i32_at(AppContext::CAMERA_X)?;
                    let width = get_drawable_width(ctx)?;
                    let instance = ctx
                        .bg_effects
                        .instances
                        .get(index)
                        .ok_or(Fault::out_of_range())?;
                    let placed = origin + spacing.wrapping_mul(step) as f32
                        - ops::div_10(camera) as f32;
                    let x = ops::cvttss2si(
                        ops::div_2(width.wrapping_add(-0x3c0)) as f32 + placed,
                    );
                    let y = ops::cvttss2si(instance.y);

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

    Ok(())
}
