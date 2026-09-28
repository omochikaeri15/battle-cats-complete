use std::rc::Rc;

use crate::Fault;

use super::{AppContext, Imgcut, scene_img008_load, texture_cache_load};

pub fn ui_sheet_cached(ctx: &mut AppContext, id: i32) -> Result<Option<Rc<Imgcut>>, Fault> {
    let slot = id as i64 as usize;

    if (id as u32) <= 0xf
        && let Some(sheet) = ctx.ui_sheet_cache.get(slot).and_then(|held| held.clone())
    {
        return Ok(Some(sheet));
    }

    let loaded = match id {
        0 => texture_cache_load(ctx, b"img006.png", b"img006.imgcut", 0x2601)?,
        1 => texture_cache_load(ctx, b"img007.png", b"img007.imgcut", 0x2601)?,
        2 => scene_img008_load(ctx)?,
        3 => texture_cache_load(ctx, b"img009.png", b"img009.imgcut", 0x2601)?,
        4 => texture_cache_load(ctx, b"img010.png", b"img010.imgcut", 0x2601)?,
        5 => texture_cache_load(ctx, b"img024.png", b"img024.imgcut", 0x2601)?,
        0xf => texture_cache_load(ctx, b"img002.png", b"img002.imgcut", 0x2601)?,
        0x15 => texture_cache_load(ctx, b"img006.png", b"img006.imgcut", 0x2601)?,
        _ => return ctx.ui_sheet_cache.get(slot).cloned().ok_or(Fault::index_out_of_range(slot as i64, ctx.ui_sheet_cache.len() as i64)),
    };

    *ctx.ui_sheet_cache.get_mut(slot).ok_or(Fault::index_out_of_range(slot as i64, 0x16))? = loaded;

    ctx.ui_sheet_cache.get(slot).cloned().ok_or(Fault::index_out_of_range(slot as i64, 0x16))
}
