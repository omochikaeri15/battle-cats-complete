use std::{cell::Cell, rc::Rc};

use crate::{Fault, ops};

use super::{
    bg_param_resolve_int, draw_context, draw_cut, draw_cut_rotated, draw_cut_scaled, draw_model, fill_rect, get_background_id, get_drawable_width, glow_set,
    imgcut_get_sprite_cut, maanim_execute, mamodel_get_angle_unit, mamodel_get_opacity_unit, mamodel_get_part, mamodel_get_scale_unit, mamodel_set_sheet,
    mamodel_set_sheet_table, set_alpha, set_part_angle, set_part_opacity, set_part_scale, set_tint, set_tint_alpha, sin_deg, AppContext,
};

const PARTICLE_RED: [i32; 6] = [255, 180, 180, 255, 255, 180];
const PARTICLE_GREEN: [i32; 6] = [180, 255, 180, 255, 180, 255];
const PARTICLE_BLUE: [i32; 6] = [180, 180, 255, 180, 255, 255];

pub fn draw_background_effects(ctx: &mut AppContext) -> Result<(), Fault> {
    set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    if get_background_id(ctx)? == 2
        || get_background_id(ctx)? == 0xe
        || get_background_id(ctx)? == 0x1a
        || get_background_id(ctx)? == 0x1b
        || get_background_id(ctx)? == 0x22
        || get_background_id(ctx)? == 0x432
    {
        for particle in 0..100usize {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let depth = ctx.i32_at(record + 8)?;

            if depth > 0x190 {
                continue;
            }

            let kind = ctx.i32_at(record + 0x10)?;
            let mut lit = true;

            if kind != 0 && get_background_id(ctx)? != 0xe {
                let shade = kind.wrapping_add(-1);

                if (shade as u32) > 5 {
                    lit = false;
                } else {
                    let shade = shade as i64 as usize;

                    ctx.set_i32_at(AppContext::DRAW_TEMP_0, PARTICLE_RED[shade])?;
                    ctx.set_i32_at(AppContext::DRAW_TEMP_1, PARTICLE_GREEN[shade])?;
                    ctx.set_i32_at(AppContext::DRAW_TEMP_2, PARTICLE_BLUE[shade])?;
                }
            } else {
                ctx.set_i32_at(AppContext::DRAW_TEMP_0, 0xff)?;
                ctx.set_i32_at(AppContext::DRAW_TEMP_1, 0xff)?;
                ctx.set_i32_at(AppContext::DRAW_TEMP_2, 0xff)?;
            }

            if lit {
                let fade = ops::cvttss2si(sin_deg(depth as f32 * 0.45) * 255.0);

                ctx.set_i32_at(AppContext::DRAW_TEMP_3, fade)?;
            }

            let red = ctx.i32_at(AppContext::DRAW_TEMP_0)?;
            let green = ctx.i32_at(AppContext::DRAW_TEMP_1)?;
            let blue = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

            set_tint(draw_context(&mut ctx.draw)?, red, green, blue, 0xff);

            let alpha = ctx.i32_at(AppContext::DRAW_TEMP_3)?;

            set_tint_alpha(draw_context(&mut ctx.draw)?, alpha);

            let drift = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?).wrapping_add(ctx.i32_at(record)?);
            let x = drift.wrapping_sub(get_drawable_width(ctx)?).wrapping_add(0x3c0);
            let y = ctx.i32_at(record + 4)?;

            fill_rect(draw_context(&mut ctx.draw)?, x, y, 4, 4);
            set_tint_alpha(draw_context(&mut ctx.draw)?, 0xff);
        }
    }

    if get_background_id(ctx)? == 0x21 || get_background_id(ctx)? == 0x3a {
        set_alpha(draw_context(&mut ctx.draw)?, 0x32);

        for glint in 0..0x1eusize {
            let record = AppContext::BG_GLINTS.wrapping_add(glint * 0x10);
            let sheet = ctx.effect_a_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let x = ctx.i32_at(record)?;
            let y = ctx.i32_at(record + 4)?;

            draw_cut(draw_context(&mut ctx.draw)?, sheet, x, y, 0x1c);
        }

        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
    }

    if get_background_id(ctx)? == 0x29 || get_background_id(ctx)? == 0x4b || get_background_id(ctx)? == 0x3f0 {
        for sprite in 0..100usize {
            let base = AppContext::BG_SPRITES.wrapping_add(0x18).wrapping_add(sprite * 0x40);
            let fade = ctx.i32_at(base)?;

            if fade <= 0x1d {
                set_alpha(draw_context(&mut ctx.draw)?, ops::div_30((fade << 8).wrapping_sub(fade)));
            }

            let sheet = ctx.bg_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let left = ops::div_4(ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?)).wrapping_add(ops::div_100(ctx.i32_at(base + 8)?));
            let x = left.wrapping_sub(get_drawable_width(ctx)?).wrapping_add(0x3c4);
            let y = ops::div_100(ctx.i32_at(base + 0xc)?);
            let cut = ctx.i32_at(base + 0x14)?;
            let span = ops::div_100(ctx.i32_at(base + 0x18)?.wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[2]));
            let cut = ctx.i32_at(base + 0x14)?;
            let height = ops::div_100(ctx.i32_at(base + 0x18)?.wrapping_mul(imgcut_get_sprite_cut(sheet, cut)?[3]));
            let cut = ctx.i32_at(base + 0x14)?;
            let angle = ctx.i32_at(base + 0x20)? as f32;

            draw_cut_rotated(draw_context(&mut ctx.draw)?, sheet, x, y, span, height, angle, 5, 0, 0, 5, cut);
            set_alpha(draw_context(&mut ctx.draw)?, 0xff);
        }
    }

    if get_background_id(ctx)? == 0x37 {
        glow_set(draw_context(&mut ctx.draw)?, 1);

        for spark in 0..100usize {
            let record = AppContext::BG_PARTICLES.wrapping_add(8).wrapping_add(spark * 0x14);
            let life = ctx.i32_at(record)?;

            if life < 0 {
                continue;
            }

            let scale = if life > 2 {
                let raised = life.wrapping_mul(0x64);
                let shifted = raised.wrapping_add(-0x12c);
                let folded = if shifted >= 0 { shifted } else { raised.wrapping_add(-0x125) };

                0x64i32.wrapping_sub(folded >> 3)
            } else {
                ops::mul_high(life.wrapping_mul(0x64) as u8 as i32, 0xab) >> 9
            };
            let sheet = ctx.bg_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let span = ops::div_100(imgcut_get_sprite_cut(sheet, 0x14)?[2].wrapping_mul(scale));
            let height = ops::div_100(imgcut_get_sprite_cut(sheet, 0x14)?[3].wrapping_mul(scale));
            let drift = ops::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?);
            let x = ops::div_neg_200(imgcut_get_sprite_cut(sheet, 0x14)?[2].wrapping_mul(scale))
                .wrapping_add(ctx.i32_at(record + 4)?)
                .wrapping_add(drift)
                .wrapping_sub(get_drawable_width(ctx)?)
                .wrapping_add(0x3c0);
            let y = ops::div_neg_200(imgcut_get_sprite_cut(sheet, 0x14)?[3].wrapping_mul(scale)).wrapping_add(ctx.i32_at(record + 8)?);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, span, height, 0x14);
        }

        glow_set(draw_context(&mut ctx.draw)?, 0);
    }

    if ctx.bg_effects.instances.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    loop {
        let instance = &ctx.bg_effects.instances[index];

        if instance.z == 2 && instance.wait <= 0 {
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
