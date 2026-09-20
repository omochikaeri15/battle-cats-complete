use crate::{Fault, operation};

use super::{
    AppContext, Surface, draw_context, draw_surface_scaled, get_castle_row, get_drawable_width,
    imgcut_get_height, imgcut_get_width, read_flag,
};

pub fn draw_castle(ctx: &mut AppContext, faction: i32, x: i32, y: i32) -> Result<(), Fault> {
    let x =
        operation::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x as f64);
    let flags = read_flag(ctx, AppContext::faction_flags(faction))?;

    if flags & 1 != 0 {
        let top = y.wrapping_add(0xa3);
        let sheet = ctx
            .base_sheets
            .get(3)
            .ok_or(Fault::index_out_of_range(3, ctx.base_sheets.len() as i64))?
            .take();

        ctx.base_sheets[3].set(sheet.clone());

        let decor = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let width = imgcut_get_width(decor);
        let width = operation::div_100((width << 7).wrapping_sub(width));
        let height = imgcut_get_height(decor);
        let height = operation::div_100((height << 7).wrapping_sub(height));

        draw_surface_scaled(
            draw_context(&mut ctx.draw)?,
            Surface::Sheet(decor),
            x,
            top,
            width,
            height,
        );

        let sheet = ctx.base_sheets[2].take();

        ctx.base_sheets[2].set(sheet.clone());

        let foundation = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let width = imgcut_get_width(foundation);
        let width = operation::div_100((width << 7).wrapping_sub(width));
        let height = imgcut_get_height(foundation);
        let height = operation::div_100((height << 7).wrapping_sub(height));

        draw_surface_scaled(
            draw_context(&mut ctx.draw)?,
            Surface::Sheet(foundation),
            x,
            top,
            width,
            height,
        );

        let sheet = ctx.base_sheets[0].take();

        ctx.base_sheets[0].set(sheet.clone());

        let cannon = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let width = imgcut_get_width(cannon);
        let width = operation::div_100((width << 7).wrapping_sub(width));
        let height = imgcut_get_height(cannon);
        let height = operation::div_100((height << 7).wrapping_sub(height));

        draw_surface_scaled(
            draw_context(&mut ctx.draw)?,
            Surface::Sheet(cannon),
            x,
            y,
            width,
            height,
        );

        return Ok(());
    }

    if flags & 2 == 0 {
        return Ok(());
    }

    let sheet = ctx.enemy_castle_sheet.clone();
    let castle = sheet.as_deref().ok_or(Fault::null_pointer())?;
    let id = if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
        || ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63
    {
        ctx.i32_at(AppContext::STAGE_CASTLE_ID)?
    } else {
        ctx.i32_at(AppContext::CASTLE_ID)?
    };
    let size = get_castle_row(&ctx.enemy_castle, id)?.size;
    let width = operation::div_100((size << 7).wrapping_sub(size));
    let id = if ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
        || ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63
    {
        ctx.i32_at(AppContext::STAGE_CASTLE_ID)?
    } else {
        ctx.i32_at(AppContext::CASTLE_ID)?
    };
    let size = get_castle_row(&ctx.enemy_castle, id)?.size;
    let height = operation::div_100((size << 8).wrapping_sub(size));

    draw_surface_scaled(
        draw_context(&mut ctx.draw)?,
        Surface::Sheet(castle),
        x,
        y,
        width,
        height,
    );

    Ok(())
}
