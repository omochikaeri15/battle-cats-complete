use crate::{Fault, operation};

use super::{
    AppContext, draw_context, draw_cut, draw_cut_scaled, draw_number_scaled,
    get_left_inset_logical, get_money, get_stage_record, get_top_inset_offset, get_worker_level,
    get_worker_upgrade_cost,
};

const SITE: &str = "draw_worker_cat";

pub fn draw_worker_cat(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_stage_record(ctx, -2, 0, 0, 0, 0)? <= 0 {
        return Ok(());
    }

    let wallet = AppContext::faction_flags(0);

    if get_worker_level(ctx, wallet)? == 7 {
        let x = ctx.i32_at(AppContext::WORKER_RECT)?;
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::WORKER_RECT + 4)?);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x,
            y,
            6,
        );

        let x = get_left_inset_logical(ctx).wrapping_add(4);
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::WORKER_RECT + 4)?)
            .wrapping_add(0x5c);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img001_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x,
            y,
            0x51,
        );

        let x = get_left_inset_logical(ctx).wrapping_add(4);
        let row = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = get_top_inset_offset(ctx)
            .wrapping_add(row)
            .wrapping_add(0x1fe);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img001_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x,
            y,
            0xd,
        );

        let x = get_left_inset_logical(ctx);
        let row = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let top = get_top_inset_offset(ctx);
        let level = get_worker_level(ctx, wallet)?;

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img001_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x.wrapping_add(0x49),
            top.wrapping_add(row).wrapping_add(0x1fe),
            level.wrapping_add(0xf),
        );

        return Ok(());
    }

    let money = get_money(ctx, wallet)?;
    let affordable = money >= get_worker_upgrade_cost(ctx, wallet)?;

    let (base, label, level_cut) = if !affordable {
        let x = ctx.i32_at(AppContext::WORKER_RECT)?;
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::WORKER_RECT + 4)?);

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x,
            y,
            5,
        );

        (0x2e, 0x18, 0x1a)
    } else {
        let y = ctx
            .i32_at(AppContext::DECK_BAR_SLIDE)?
            .wrapping_add(ctx.i32_at(AppContext::WORKER_RECT + 4)?);
        let cut = if ctx.i32_at(AppContext::BLINK_ON)? == 0 {
            6
        } else {
            0x18
        };
        let x = ctx.i32_at(AppContext::WORKER_RECT)?;

        draw_cut(
            draw_context(&mut ctx.draw)?,
            ctx.img002_sheet
                .as_deref()
                .ok_or(Fault::NullPointer { site: SITE })?,
            x,
            y,
            cut,
        );

        (0x23, 0xd, 0xf)
    };

    let cost = operation::div_100(get_worker_upgrade_cost(ctx, wallet)?);
    let left = get_left_inset_logical(ctx).wrapping_add(4);
    let y = ctx
        .i32_at(AppContext::DECK_BAR_SLIDE)?
        .wrapping_add(ctx.i32_at(AppContext::WORKER_RECT + 4)?)
        .wrapping_add(0x5c);
    let area = draw_number_scaled(
        draw_context(&mut ctx.draw)?,
        ctx.img001_sheet
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
        base,
        cost,
        0,
        left as f32,
        y as f32,
        -3.0,
        1.0,
        0x16,
        0,
        0,
    )?;

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        ctx.img001_sheet
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
        operation::cvttss2si(area.left),
        y,
        0x16,
        0x1a,
        0x38,
    );

    let x = get_left_inset_logical(ctx).wrapping_add(4);
    let row = ctx
        .i32_at(AppContext::DECK_BAR_SLIDE)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let y = get_top_inset_offset(ctx)
        .wrapping_add(row)
        .wrapping_add(0x1fe);

    draw_cut(
        draw_context(&mut ctx.draw)?,
        ctx.img001_sheet
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
        x,
        y,
        label,
    );

    let x = get_left_inset_logical(ctx).wrapping_add(0x49);
    let row = ctx
        .i32_at(AppContext::DECK_BAR_SLIDE)?
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let y = get_top_inset_offset(ctx)
        .wrapping_add(row)
        .wrapping_add(0x1fe);
    let level = get_worker_level(ctx, wallet)?;

    draw_cut(
        draw_context(&mut ctx.draw)?,
        ctx.img001_sheet
            .as_deref()
            .ok_or(Fault::NullPointer { site: SITE })?,
        x,
        y,
        level.wrapping_add(level_cut),
    );

    Ok(())
}
