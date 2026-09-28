use crate::Fault;

use super::{
    AppContext, ad_is_showing, batch_tasks_pending, cat_food_shop_draw, dc_begin_frame, dc_end_frame,
    dc_set_viewport, dialog_manager_draw, draw_context, draw_screen_transition, fill_rect,
    get_design_height2, get_drawable_width, get_left_inset_logical, gl_surface_ready,
    inquiry_button_draw, main_draw, medal_popup_draw, mission_popup_draw, notice_popup_draw,
    set_draw_origin, set_draw_scale, set_tint, surface_height, surface_width, tutorial_popup_draw,
};

pub fn app_on_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    if gl_surface_ready(ctx) {
        dc_begin_frame(draw_context(&mut ctx.draw)?);
        ctx.set_block_at::<0x28>(AppContext::DRAW_TEMP_0, [0; 0x28])?;

        let surface_w = surface_width(ctx);
        let surface_h = surface_height(ctx);
        let screen_w = ctx.screen_metrics.screen_w;
        let screen_h = ctx.screen_metrics.screen_h;

        dc_set_viewport(draw_context(&mut ctx.draw)?, surface_w, surface_h, screen_w, screen_h, 100.0);

        let scale = ctx.screen_metrics.scale2;

        set_draw_scale(draw_context(&mut ctx.draw)?, scale);

        let lifted = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        set_draw_origin(ctx, 0, lifted)?;
        ctx.set_block_at::<1>(AppContext::INSET_STRIPS_DUE, [1])?;

        if !ad_is_showing(ctx)? {
            let mut loading = false;

            if batch_tasks_pending(ctx)? {
                let scene = ctx.i32_at(AppContext::DRAW_SCENE_ID)?;

                if scene == 0x65 || scene == 0x61 {
                    loading = !ctx.scene_host().ok_or(Fault::host_missing())?.loader_resuming_sound();
                }
            }

            if !loading {
                let mut scene = ctx.i32_at(AppContext::DRAW_SCENE_ID)?;

                if scene | 4 == 0x65 {
                    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
                    let width = get_drawable_width(ctx)?;
                    let height = get_design_height2(ctx);

                    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                    fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);

                    scene = ctx.i32_at(AppContext::DRAW_SCENE_ID)?;
                }

                if scene > 0x59 {
                    if (scene.wrapping_sub(0x5a) as u32) <= 0xc {
                        ctx.scene_host().ok_or(Fault::host_missing())?.draw_menu_scene(scene);
                    }
                }

                if scene == 0x12c {
                    let hidden = ctx.u8_at(AppContext::MAIN_DRAW_HIDDEN)?;

                    main_draw(ctx, hidden)?;
                }

                if scene == 0x3e7 {
                    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
                    let width = get_drawable_width(ctx)?;
                    let height = get_design_height2(ctx);

                    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                    fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);

                    let style = ctx.u8_at(AppContext::CURTAIN_STYLE)? as i32;

                    draw_screen_transition(ctx, style)?;
                }

                if scene == 4 || scene == 5 {
                    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
                    let width = get_drawable_width(ctx)?;
                    let height = get_design_height2(ctx);

                    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                    fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);
                    inquiry_button_draw(ctx)?;
                }
            } else {
                let scene = ctx.i32_at(AppContext::DRAW_SCENE_ID)?;

                ctx.scene_host().ok_or(Fault::host_missing())?.draw_loading_scene(scene);
            }

            set_draw_scale(draw_context(&mut ctx.draw)?, scale);
            cat_food_shop_draw(ctx)?;
            notice_popup_draw(ctx)?;

            if ctx.u8_at(AppContext::TUTORIAL_POPUP_OPEN)? != 0 {
                tutorial_popup_draw(ctx)?;
            }

            dialog_manager_draw(ctx)?;

            if ctx.i32_at(AppContext::DRAW_SCENE_ID)? == 0x64
                && ctx.i32_at(AppContext::SCENE_0X64_PAGE)? == 9
                && ctx.u8_at(AppContext::MAP_STAMINA_HUD_STATE)? == 3
            {
                ctx.scene_host().ok_or(Fault::host_missing())?.draw_map_stamina_hud();
            }

            let pad = ctx.i32_at(AppContext::LETTERBOX_PAD)?;

            set_draw_origin(ctx, 0, pad)?;
            medal_popup_draw(ctx)?;
            mission_popup_draw(ctx)?;
            set_draw_origin(ctx, 0, 0)?;

            if ctx.i32_at(AppContext::LETTERBOX_PAD)? > 0 {
                let left = 0i32.wrapping_sub(get_left_inset_logical(ctx));
                let width = ctx.screen_metrics.screen_w;
                let pad = ctx.i32_at(AppContext::LETTERBOX_PAD)?;

                set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
                fill_rect(draw_context(&mut ctx.draw)?, left, 0, width, pad);

                let left = 0i32.wrapping_sub(get_left_inset_logical(ctx));
                let pad = ctx.i32_at(AppContext::LETTERBOX_PAD)?;
                let bottom = ctx.screen_metrics.design_h2.wrapping_sub(pad);

                fill_rect(draw_context(&mut ctx.draw)?, left, bottom, width, pad);
                set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
            }
        }
    } else {
        let scene = ctx.i32_at(AppContext::SCENE_ID)?;
        let page = if scene == 0x64 { ctx.i32_at(AppContext::SCENE_0X64_PAGE)? } else { 0 };

        if scene != 0x64 || page == 0 || page == 0x1869f || ctx.u8_at(AppContext::INSETS_IGNORED)? != 0 {
            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
            fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);
        } else {
            ctx.set_block_at::<1>(AppContext::INSETS_IGNORED, [1])?;

            let lifted = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

            set_draw_origin(ctx, 0, lifted)?;

            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let width = get_drawable_width(ctx)?;
            let height = get_design_height2(ctx);

            set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
            fill_rect(draw_context(&mut ctx.draw)?, 0, 0i32.wrapping_sub(shift), width, height);
            ctx.set_block_at::<1>(AppContext::INSETS_IGNORED, [0])?;

            let lifted = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

            set_draw_origin(ctx, 0, lifted)?;
        }
    }

    dc_end_frame(draw_context(&mut ctx.draw)?);

    Ok(())
}
