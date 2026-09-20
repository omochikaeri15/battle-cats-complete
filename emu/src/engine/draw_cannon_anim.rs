use crate::{Fault, operation};

use super::{
    AppContext, CANNON_SHOT_SPACING, CannonShot, Entity, atan2_deg, cannon_reach_x, draw_context,
    draw_cut_scaled, draw_model, draw_quad_cut, draw_quad_region, get_anim_len, get_base_level,
    get_cannon_strike_width, get_cannon_strike_x, get_cannon_type, get_castle_anim_frame,
    get_castle_anim_state, get_drawable_width, glow_set, imgcut_get_sprite_cut, maanim_execute,
    mamodel_get_part, mamodel_get_scale_unit, matrix_identity, matrix_multiply,
    matrix_set_rotation, matrix_set_translation, set_alpha, set_part_scale, sqrt_f32,
    transform_point,
};

const EMBER_CUTS: [i32; 6] = [0, 0, 1, 1, 6, 6];

const SPARK_CUTS: [i32; 5] = [2, 2, 2, 3, 3];

pub fn draw_cannon_anim(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    if get_castle_anim_state(ctx, faction)? == 1 {
        let reach = cannon_reach_x(ctx, faction)?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
        let travel = get_castle_anim_frame(ctx, faction)?.wrapping_mul(step);
        let travel = (if travel >= 0 {
            travel
        } else {
            travel.wrapping_add(0xf)
        }) >> 4;
        let travel = if faction != 0 {
            travel
        } else {
            travel.wrapping_neg()
        };

        ctx.set_i32_at(AppContext::DRAW_TEMP_2, travel.wrapping_add(reach))?;

        let top =
            operation::div_10(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_Y))?)
                .wrapping_add(-0xa0);

        ctx.set_i32_at(AppContext::DRAW_TEMP_3, top)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let start_x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );
        let start_y = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
        let tip = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f64;
        let end_x =
            operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + tip);
        let angle = atan2_deg(
            0x1eai32.wrapping_sub(start_y) as f32,
            end_x.wrapping_sub(start_x) as f32,
        );
        let sheet = ctx
            .base_sheets
            .get(1)
            .ok_or(Fault::index_out_of_range(1, ctx.base_sheets.len() as i64))?
            .take();

        ctx.base_sheets[1].set(sheet.clone());

        let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let height = imgcut_get_sprite_cut(beam, 9)?[3];
        let across = start_x.wrapping_sub(end_x);
        let down = start_y.wrapping_add(-0x1ea);
        let span = across
            .wrapping_mul(across)
            .wrapping_add(down.wrapping_mul(down));
        let far = start_x.wrapping_sub(operation::cvttss2si(sqrt_f32(span as f32)));
        let half = operation::div_2(height);
        let upper = start_y.wrapping_sub(half);
        let lower = half.wrapping_add(start_y);
        let mut shift = [0.0f32; 6];
        let mut turn = [0.0f32; 6];
        let mut back = [0.0f32; 6];
        let mut mat = [0.0f32; 6];

        matrix_identity(&mut shift);
        matrix_identity(&mut turn);
        matrix_identity(&mut back);
        matrix_identity(&mut mat);
        matrix_set_translation(&mut shift, start_x, start_y);
        matrix_set_rotation(&mut turn, angle + 180.0);
        matrix_set_translation(&mut back, start_x.wrapping_neg(), start_y.wrapping_neg());
        matrix_multiply(&mut mat, &shift);
        matrix_multiply(&mut mat, &turn);
        matrix_multiply(&mut mat, &back);

        let mut first = 0i64;
        let mut second = 0i64;
        let mut third = 0i64;
        let mut fourth = 0i64;

        transform_point(&mat, far, upper, &mut first);
        transform_point(&mat, far, lower, &mut second);
        transform_point(&mat, start_x, lower, &mut third);
        transform_point(&mat, start_x, upper, &mut fourth);
        draw_quad_cut(
            draw_context(&mut ctx.draw)?,
            beam,
            first as i32,
            (first >> 32) as i32,
            second as i32,
            (second >> 32) as i32,
            third as i32,
            (third >> 32) as i32,
            fourth as i32,
            (fourth >> 32) as i32,
            9,
        );

        let sheet = ctx.base_sheets[1].take();

        ctx.base_sheets[1].set(sheet.clone());

        let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let flare = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x1e) as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + flare,
        );
        let phase = (get_castle_anim_frame(ctx, faction)?.wrapping_rem(4) as i8 / 2).wrapping_add(5)
            as u8 as i32;

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            x,
            0x1c2,
            0x84,
            0x32,
            phase,
        );
        glow_set(draw_context(&mut ctx.draw)?, 1);

        let width = imgcut_get_sprite_cut(beam, 7)?[2];
        let height = imgcut_get_sprite_cut(beam, 7)?[3];

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            start_x.wrapping_sub(operation::div_2(width)),
            start_y.wrapping_sub(operation::div_2(height)),
            width,
            height,
            7,
        );

        let width = imgcut_get_sprite_cut(beam, 7)?[2];
        let height = imgcut_get_sprite_cut(beam, 7)?[3];

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            start_x.wrapping_sub(width),
            start_y.wrapping_sub(height),
            width.wrapping_add(width),
            height.wrapping_add(height),
            7,
        );
        glow_set(draw_context(&mut ctx.draw)?, 0);
    } else if get_castle_anim_state(ctx, faction)? == 3 {
        let reach = cannon_reach_x(ctx, faction)?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
        let travel = get_castle_anim_frame(ctx, faction)?.wrapping_mul(step);
        let travel = (if travel >= 0 {
            travel
        } else {
            travel.wrapping_add(0x1f)
        }) >> 5;
        let travel = if faction != 0 {
            travel
        } else {
            travel.wrapping_neg()
        };

        ctx.set_i32_at(AppContext::DRAW_TEMP_2, travel.wrapping_add(reach))?;

        let top =
            operation::div_10(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_Y))?)
                .wrapping_add(-0xa0);

        ctx.set_i32_at(AppContext::DRAW_TEMP_3, top)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let start = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin;
        let start_y = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
        let tip = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f64;
        let end = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + tip;

        if get_castle_anim_frame(ctx, faction)? < 2 {
            set_alpha(draw_context(&mut ctx.draw)?, 0x40);
        } else if get_castle_anim_frame(ctx, faction)? >= 0x1f {
            let frame = get_castle_anim_frame(ctx, faction)?;
            let scaled = (frame << 8).wrapping_sub(frame);
            let fade = operation::mul_high(scaled.wrapping_add(-0x1de2), 0x55555555)
                .wrapping_sub(scaled)
                .wrapping_add(0x1de2);

            set_alpha(
                draw_context(&mut ctx.draw)?,
                operation::div_2(fade).wrapping_add(0xff),
            );
        }

        let start_x = operation::cvttsd2si(start);
        let end_x = operation::cvttsd2si(end);

        if get_castle_anim_frame(ctx, faction)? >= 2 {
            let angle = atan2_deg(
                0x1eai32.wrapping_sub(start_y) as f32,
                end_x.wrapping_sub(start_x) as f32,
            );
            let sheet = ctx
                .base_sheets
                .get(1)
                .ok_or(Fault::index_out_of_range(1, ctx.base_sheets.len() as i64))?
                .take();

            ctx.base_sheets[1].set(sheet.clone());

            let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let width = imgcut_get_sprite_cut(beam, 5)?[2];
            let half_width = operation::div_2(width);
            let height = imgcut_get_sprite_cut(beam, 5)?[3];
            let across = start_x.wrapping_sub(end_x);
            let down = start_y.wrapping_add(-0x1ea);
            let length = operation::cvttss2si(sqrt_f32(
                across
                    .wrapping_mul(across)
                    .wrapping_add(down.wrapping_mul(down)) as f32,
            ));
            let angle = angle + 180.0;
            let quarter = operation::div_4(width);
            let mut covered = 0i32;

            loop {
                let piece = if half_width.wrapping_add(covered) <= length {
                    half_width
                } else {
                    length.wrapping_sub(covered)
                };
                let flip = get_castle_anim_frame(ctx, faction)?.wrapping_rem(4);
                let thickness = if flip < 2 {
                    height
                } else {
                    height.wrapping_neg()
                };
                let near = start_x.wrapping_sub(covered);
                let half = operation::div_2(thickness);
                let upper = start_y.wrapping_sub(half);
                let lower = half.wrapping_add(start_y);

                covered = covered.wrapping_add(piece);

                let far = start_x.wrapping_sub(covered);
                let mut shift = [0.0f32; 6];
                let mut turn = [0.0f32; 6];
                let mut back = [0.0f32; 6];
                let mut mat = [0.0f32; 6];

                matrix_identity(&mut shift);
                matrix_identity(&mut turn);
                matrix_identity(&mut back);
                matrix_identity(&mut mat);
                matrix_set_translation(&mut shift, start_x, start_y);
                matrix_set_rotation(&mut turn, angle);
                matrix_set_translation(&mut back, start_x.wrapping_neg(), start_y.wrapping_neg());
                matrix_multiply(&mut mat, &shift);
                matrix_multiply(&mut mat, &turn);
                matrix_multiply(&mut mat, &back);

                let mut first = 0i64;
                let mut second = 0i64;
                let mut third = 0i64;
                let mut fourth = 0i64;

                transform_point(&mat, near, upper, &mut first);
                transform_point(&mat, near, lower, &mut second);
                transform_point(&mat, far, lower, &mut third);
                transform_point(&mat, far, upper, &mut fourth);

                let src_x = imgcut_get_sprite_cut(beam, 5)?[0].wrapping_add(quarter);
                let src_y = imgcut_get_sprite_cut(beam, 5)?[1];

                draw_quad_region(
                    draw_context(&mut ctx.draw)?,
                    beam,
                    first as i32,
                    (first >> 32) as i32,
                    second as i32,
                    (second >> 32) as i32,
                    third as i32,
                    (third >> 32) as i32,
                    fourth as i32,
                    (fourth >> 32) as i32,
                    src_x,
                    src_y,
                    piece,
                    height,
                );

                if covered >= length {
                    break;
                }
            }

            let index = get_castle_anim_frame(ctx, faction)?.wrapping_rem(6);
            let ember = *EMBER_CUTS
                .get(index as i64 as usize)
                .ok_or(Fault::index_out_of_range(index as i64, 6))?;
            let width = imgcut_get_sprite_cut(beam, ember)?[2].wrapping_mul(3);
            let height = imgcut_get_sprite_cut(beam, ember)?[3].wrapping_mul(3);

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                beam,
                end_x.wrapping_add(-0x50),
                0x172,
                width,
                height,
                ember,
            );
        }

        glow_set(draw_context(&mut ctx.draw)?, 1);

        let sheet = ctx
            .base_sheets
            .get(1)
            .ok_or(Fault::index_out_of_range(1, ctx.base_sheets.len() as i64))?
            .take();

        ctx.base_sheets[1].set(sheet.clone());

        let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;

        if get_castle_anim_frame(ctx, faction)? >= 2 {
            let width = imgcut_get_sprite_cut(beam, 2)?[2];
            let height = imgcut_get_sprite_cut(beam, 2)?[3].wrapping_mul(3);
            let impact = end_x.wrapping_add(5);

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                beam,
                impact.wrapping_sub(width.wrapping_mul(3)),
                0x1f9i32.wrapping_sub(height),
                width.wrapping_mul(6),
                height,
                2,
            );

            let width = imgcut_get_sprite_cut(beam, 4)?[2];
            let height = imgcut_get_sprite_cut(beam, 4)?[3];

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                beam,
                impact.wrapping_sub(width.wrapping_mul(3)),
                0x1f9,
                width.wrapping_mul(6),
                height,
                4,
            );
        }

        let width = imgcut_get_sprite_cut(beam, 3)?[2];
        let height = imgcut_get_sprite_cut(beam, 3)?[3];

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            start_x.wrapping_sub(width),
            start_y.wrapping_sub(height),
            width.wrapping_add(width),
            height.wrapping_add(height),
            3,
        );

        let width = imgcut_get_sprite_cut(beam, 3)?[2].wrapping_mul(5);
        let height = imgcut_get_sprite_cut(beam, 3)?[3].wrapping_mul(5);

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            start_x.wrapping_sub(operation::div_2(width)),
            start_y.wrapping_sub(operation::div_2(height)),
            width,
            height,
            3,
        );
        glow_set(draw_context(&mut ctx.draw)?, 0);
        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
    } else if get_castle_anim_state(ctx, faction)? == 0xc {
        let reach = cannon_reach_x(ctx, faction)?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
        let travel = get_castle_anim_frame(ctx, faction)?.wrapping_mul(step);
        let travel = (if travel >= 0 {
            travel
        } else {
            travel.wrapping_add(0x1f)
        }) >> 5;
        let travel = if faction != 0 {
            travel
        } else {
            travel.wrapping_neg()
        };

        ctx.set_i32_at(AppContext::DRAW_TEMP_2, travel.wrapping_add(reach))?;

        let top =
            operation::div_10(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_Y))?)
                .wrapping_add(-0xa0);

        ctx.set_i32_at(AppContext::DRAW_TEMP_3, top)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let near_width = get_drawable_width(ctx)?;
        let start_y = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
        let tip = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f64;
        let far_width = get_drawable_width(ctx)?;

        if get_castle_anim_frame(ctx, faction)? < 2 {
            set_alpha(draw_context(&mut ctx.draw)?, 0x40);
        } else if get_castle_anim_frame(ctx, faction)? >= 0x1f {
            let frame = get_castle_anim_frame(ctx, faction)?;
            let scaled = (frame << 8).wrapping_sub(frame);
            let fade = operation::mul_high(scaled.wrapping_add(-0x1de2), 0x55555555)
                .wrapping_sub(scaled)
                .wrapping_add(0x1de2);

            set_alpha(
                draw_context(&mut ctx.draw)?,
                operation::div_2(fade).wrapping_add(0xff),
            );
        }

        if get_castle_anim_frame(ctx, faction)? >= 2 {
            let start_x =
                operation::cvttsd2si(near_width.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
            let end_x = operation::cvttsd2si(far_width.wrapping_add(-0x3c0) as f64 * 0.5 + tip);
            let angle = atan2_deg(
                0x1eai32.wrapping_sub(start_y) as f32,
                end_x.wrapping_sub(start_x) as f32,
            );
            let sheet = ctx
                .base_sheets
                .get(1)
                .ok_or(Fault::index_out_of_range(1, ctx.base_sheets.len() as i64))?
                .take();

            ctx.base_sheets[1].set(sheet.clone());

            let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;
            let width = imgcut_get_sprite_cut(beam, 5)?[2];
            let height = imgcut_get_sprite_cut(beam, 5)?[3];
            let across = start_x.wrapping_sub(end_x);
            let down = start_y.wrapping_add(-0x1ea);
            let length = operation::cvttss2si(sqrt_f32(
                across
                    .wrapping_mul(across)
                    .wrapping_add(down.wrapping_mul(down)) as f32,
            ));
            let mut seed = operation::div_2(get_castle_anim_frame(ctx, faction)?);
            let half = operation::div_2(height);
            let upper = start_y.wrapping_sub(half);
            let lower = half.wrapping_add(start_y);
            let angle = angle + 180.0;
            let mut covered = 0i32;

            loop {
                let piece = if width.wrapping_add(covered) <= length {
                    width
                } else {
                    length.wrapping_sub(covered)
                };
                let near = start_x.wrapping_sub(covered);

                covered = covered.wrapping_add(piece);

                let far = start_x.wrapping_sub(covered);
                let mut shift = [0.0f32; 6];
                let mut turn = [0.0f32; 6];
                let mut back = [0.0f32; 6];
                let mut mat = [0.0f32; 6];

                matrix_identity(&mut shift);
                matrix_identity(&mut turn);
                matrix_identity(&mut back);
                matrix_identity(&mut mat);
                matrix_set_translation(&mut shift, start_x, start_y);
                matrix_set_rotation(&mut turn, angle);
                matrix_set_translation(&mut back, start_x.wrapping_neg(), start_y.wrapping_neg());
                matrix_multiply(&mut mat, &shift);
                matrix_multiply(&mut mat, &turn);
                matrix_multiply(&mut mat, &back);

                let mut first = 0i64;
                let mut second = 0i64;
                let mut third = 0i64;
                let mut fourth = 0i64;

                transform_point(&mat, near, upper, &mut first);
                transform_point(&mat, near, lower, &mut second);
                transform_point(&mat, far, lower, &mut third);
                transform_point(&mat, far, upper, &mut fourth);

                let cut = (seed & 3).wrapping_add(3);
                let mixed = seed ^ (seed << 13);
                let mixed = ((mixed as u32 >> 17) as i32) ^ mixed;

                seed = (mixed << 15) ^ mixed;

                let src_x = imgcut_get_sprite_cut(beam, cut)?[0];
                let src_y = imgcut_get_sprite_cut(beam, cut)?[1];

                draw_quad_region(
                    draw_context(&mut ctx.draw)?,
                    beam,
                    first as i32,
                    (first >> 32) as i32,
                    second as i32,
                    (second >> 32) as i32,
                    third as i32,
                    (third >> 32) as i32,
                    fourth as i32,
                    (fourth >> 32) as i32,
                    src_x,
                    src_y,
                    piece,
                    height,
                );

                if covered >= length {
                    break;
                }
            }

            let frame = get_castle_anim_frame(ctx, faction)?;

            maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;
            draw_model(
                draw_context(&mut ctx.draw)?,
                &ctx.base_models[0],
                end_x.wrapping_add(0x96),
                0x1e0,
            );
            glow_set(draw_context(&mut ctx.draw)?, 1);

            let width = imgcut_get_sprite_cut(beam, 1)?[2];
            let height = imgcut_get_sprite_cut(beam, 1)?[3];

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                beam,
                start_x.wrapping_sub(width),
                start_y.wrapping_sub(height),
                width.wrapping_add(width),
                height.wrapping_add(height),
                1,
            );
            glow_set(draw_context(&mut ctx.draw)?, 0);
        }

        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
    } else if get_castle_anim_state(ctx, faction)? == 9 {
        let reach = cannon_reach_x(ctx, faction)?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
        let travel = get_castle_anim_frame(ctx, faction)?.wrapping_mul(step);
        let travel = (if travel >= 0 {
            travel
        } else {
            travel.wrapping_add(0xf)
        }) >> 4;
        let travel = if faction != 0 {
            travel
        } else {
            travel.wrapping_neg()
        };

        ctx.set_i32_at(AppContext::DRAW_TEMP_2, travel.wrapping_add(reach))?;

        let top =
            operation::div_10(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_Y))?)
                .wrapping_add(-0xa0);

        ctx.set_i32_at(AppContext::DRAW_TEMP_3, top)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let start_x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );
        let start_y = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
        let tip = ctx.i32_at(AppContext::DRAW_TEMP_2)? as f64;
        let end_x =
            operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + tip);

        if get_castle_anim_frame(ctx, faction)? < 2 {
            set_alpha(draw_context(&mut ctx.draw)?, 0x40);
        } else if get_castle_anim_frame(ctx, faction)? >= 0x1f {
            let frame = get_castle_anim_frame(ctx, faction)?;
            let scaled = (frame << 8).wrapping_sub(frame);
            let fade = operation::mul_high(scaled.wrapping_add(-0x1de2), 0x55555555)
                .wrapping_sub(scaled)
                .wrapping_add(0x1de2);

            set_alpha(
                draw_context(&mut ctx.draw)?,
                operation::div_2(fade).wrapping_add(0xff),
            );
        }

        let angle = atan2_deg(
            0x1eai32.wrapping_sub(start_y) as f32,
            end_x.wrapping_sub(start_x) as f32,
        );
        let sheet = ctx
            .base_sheets
            .get(1)
            .ok_or(Fault::index_out_of_range(1, ctx.base_sheets.len() as i64))?
            .take();

        ctx.base_sheets[1].set(sheet.clone());

        let beam = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let height = imgcut_get_sprite_cut(beam, 4)?[3];
        let across = start_x.wrapping_sub(end_x);
        let down = start_y.wrapping_add(-0x1ea);
        let far = start_x.wrapping_sub(operation::cvttss2si(sqrt_f32(
            across
                .wrapping_mul(across)
                .wrapping_add(down.wrapping_mul(down)) as f32,
        )));
        let half = operation::div_2(height);
        let upper = start_y.wrapping_sub(half);
        let lower = half.wrapping_add(start_y);
        let mut shift = [0.0f32; 6];
        let mut turn = [0.0f32; 6];
        let mut back = [0.0f32; 6];
        let mut mat = [0.0f32; 6];

        matrix_identity(&mut shift);
        matrix_identity(&mut turn);
        matrix_identity(&mut back);
        matrix_identity(&mut mat);
        matrix_set_translation(&mut shift, start_x, start_y);
        matrix_set_rotation(&mut turn, angle + 180.0);
        matrix_set_translation(&mut back, start_x.wrapping_neg(), start_y.wrapping_neg());
        matrix_multiply(&mut mat, &shift);
        matrix_multiply(&mut mat, &turn);
        matrix_multiply(&mut mat, &back);

        let mut first = 0i64;
        let mut second = 0i64;
        let mut third = 0i64;
        let mut fourth = 0i64;

        transform_point(&mat, far, upper, &mut first);
        transform_point(&mat, far, lower, &mut second);
        transform_point(&mat, start_x, lower, &mut third);
        transform_point(&mat, start_x, upper, &mut fourth);
        draw_quad_cut(
            draw_context(&mut ctx.draw)?,
            beam,
            first as i32,
            (first >> 32) as i32,
            second as i32,
            (second >> 32) as i32,
            third as i32,
            (third >> 32) as i32,
            fourth as i32,
            (fourth >> 32) as i32,
            4,
        );

        let index = get_castle_anim_frame(ctx, faction)?.wrapping_rem(5);
        let spark = *SPARK_CUTS
            .get(index as i64 as usize)
            .ok_or(Fault::index_out_of_range(index as i64, 5))?;
        let width = imgcut_get_sprite_cut(beam, spark)?[2];
        let height = imgcut_get_sprite_cut(beam, spark)?[3];

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            beam,
            end_x.wrapping_add(-0x1e),
            0x1c2,
            width,
            height,
            spark,
        );

        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;
        draw_model(
            draw_context(&mut ctx.draw)?,
            &ctx.base_models[0],
            start_x,
            start_y,
        );
        set_alpha(draw_context(&mut ctx.draw)?, 0xff);
    } else if get_castle_anim_state(ctx, faction)? == 4 {
        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[1], Some(&ctx.base_anims[1]), frame, 0)?;

        let pos = ctx
            .i32_at(AppContext::entity_field(faction, 0, Entity::POS_X))?
            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
        let origin = operation::div_10(pos).wrapping_add(0x55) as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );

        draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[1], x, 0x82);
    } else if get_castle_anim_state(ctx, faction)? == 5 {
        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;

        let origin = operation::div_10(
            get_cannon_strike_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        ) as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );

        draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[0], x, 0x1d6);
    } else if get_castle_anim_state(ctx, faction)? == 0xb {
        if get_castle_anim_frame(ctx, faction)? >= 5 {
            let frame = get_castle_anim_frame(ctx, faction)?.wrapping_add(-5);

            maanim_execute(&mut ctx.base_models[1], Some(&ctx.base_anims[1]), frame, 0)?;

            let reach = cannon_reach_x(ctx, faction)?;
            let x = reach
                .wrapping_add(operation::div_2(
                    get_drawable_width(ctx)?.wrapping_add(-0x3c0),
                ))
                .wrapping_add(0x14);
            let y = operation::div_10(ctx.i32_at(AppContext::entity_field(
                faction,
                0,
                Entity::POS_Y,
            ))?)
            .wrapping_add(-0xa0);

            draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[1], x, y);
        }

        if get_castle_anim_frame(ctx, faction)? >= 0xa {
            let frame = get_castle_anim_frame(ctx, faction)?.wrapping_add(-0xa);

            maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;

            let origin = operation::div_10(
                get_cannon_strike_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
            ) as f64;
            let x = operation::cvttsd2si(
                get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
            );

            draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[0], x, 0x1f4);

            let frame = get_castle_anim_frame(ctx, faction)?.wrapping_add(-0xa);

            maanim_execute(&mut ctx.base_models[2], Some(&ctx.base_anims[2]), frame, 0)?;

            let origin = operation::div_10(
                get_cannon_strike_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
            ) as f64;
            let x = operation::cvttsd2si(
                get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
            );

            draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[2], x, 0x1f4);

            let part = mamodel_get_part(&ctx.base_models[2], 0)
                .ok_or(Fault::null_pointer())?;
            let unit = mamodel_get_scale_unit(&ctx.base_models[2]);
            let stretch = operation::div_1600(get_cannon_strike_width(ctx, 0)?.wrapping_mul(unit));
            let unit = mamodel_get_scale_unit(&ctx.base_models[2]);

            set_part_scale(&mut ctx.base_models[2].parts[part], stretch, unit);
        }
    } else if get_castle_anim_state(ctx, faction)? == 7 {
        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[1], Some(&ctx.base_anims[1]), frame, 0)?;

        let pos = ctx
            .i32_at(AppContext::entity_field(faction, 0, Entity::POS_X))?
            .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
        let origin = operation::div_10(pos).wrapping_add(0x55) as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );

        draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[1], x, 0x82);
    } else if get_castle_anim_state(ctx, faction)? == 8 {
        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;

        let origin = operation::div_10(
            get_cannon_strike_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        ) as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin,
        );

        draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[0], x, 0x1d6);
    } else if get_castle_anim_state(ctx, faction)? == 6 {
        let reach = cannon_reach_x(ctx, faction)?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

        let top =
            operation::div_10(ctx.i32_at(AppContext::entity_field(faction, 0, Entity::POS_Y))?)
                .wrapping_add(-0xa0);

        ctx.set_i32_at(AppContext::DRAW_TEMP_3, top)?;

        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5
                + reach.wrapping_add(-10) as f64,
        );
        let y = ctx.i32_at(AppContext::DRAW_TEMP_3)?.wrapping_add(5);
        let frame = get_castle_anim_frame(ctx, faction)?;

        maanim_execute(&mut ctx.base_models[0], Some(&ctx.base_anims[0]), frame, 0)?;
        draw_model(draw_context(&mut ctx.draw)?, &ctx.base_models[0], x, y);
    }

    let shots = AppContext::CANNON_SHOTS.wrapping_add(
        (faction as i64 as usize).wrapping_mul(AppContext::CANNON_SHOTS_FACTION_STRIDE),
    );

    for shot in 0..0xfusize {
        let record = shots.wrapping_add(shot.wrapping_mul(AppContext::CANNON_SHOT_STRIDE));

        if ctx.i32_at(record.wrapping_add(CannonShot::TIMER))? == 0 {
            continue;
        }

        let pos = operation::div_10(
            ctx.i32_at(record.wrapping_add(CannonShot::POS_X))?
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?),
        );

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos)?;

        let (length, model, scale) = if get_cannon_type(ctx, faction)? == 0 {
            (0xb, 0i64, 2.5f32)
        } else if get_cannon_type(ctx, faction)? == 5 {
            (get_anim_len(&ctx.base_anims[1])?, 1, 1.0)
        } else {
            (0, -1, 1.0)
        };

        if model < 0 {
            return Err(Fault::unrepresentable("a live cannon shot on a cannon type that never fires shots"));
        }

        let frame = length.wrapping_sub(ctx.i32_at(record.wrapping_add(CannonShot::TIMER))?);
        let model = model as usize;

        maanim_execute(
            &mut ctx.base_models[model],
            Some(&ctx.base_anims[model]),
            frame,
            0,
        )?;

        let part = mamodel_get_part(&ctx.base_models[model], 0)
            .ok_or(Fault::null_pointer())?;
        let width =
            operation::cvttss2si(mamodel_get_scale_unit(&ctx.base_models[model]) as f32 * scale);
        let height =
            operation::cvttss2si(mamodel_get_scale_unit(&ctx.base_models[model]) as f32 * scale);

        set_part_scale(&mut ctx.base_models[model].parts[part], width, height);

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let x = operation::cvttsd2si(
            get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5
                + origin
                + operation::div_2(CANNON_SHOT_SPACING) as f64,
        );

        draw_model(
            draw_context(&mut ctx.draw)?,
            &ctx.base_models[model],
            x,
            0x1f4,
        );
    }

    Ok(())
}
