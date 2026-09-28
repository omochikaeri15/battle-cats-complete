use std::rc::Rc;

use crate::Fault;

use super::{AppContext, Imgcut, labyrinth_active, query_localizable, texture_cache_load};

pub fn scene_img008_load(ctx: &mut AppContext) -> Result<Option<Rc<Imgcut>>, Fault> {
    if !labyrinth_active(ctx)? {
        if ctx.i32_at(AppContext::CHAPTER_MODE)? != 3 || ctx.u8_at(AppContext::SCORE_MODE_FLAG)? == 0 {
            let cut = query_localizable(ctx, b"img008.imgcut");

            return texture_cache_load(ctx, b"img008.png", &cut, 0x2601);
        }

        let cut = query_localizable(ctx, b"img008.imgcut");

        return texture_cache_load(ctx, b"img008_nekoDojo.png", &cut, 0x2601);
    }

    let cut = query_localizable(ctx, b"img008.imgcut");

    texture_cache_load(ctx, b"img008_Labyrinth.png", &cut, 0x2601)
}
