use crate::{
    Fault,
    engine::{AppContext, texture_cache_load},
};

const POPUP_PNG: &[u8] = b"img005.png";
const POPUP_CUT: &[u8] = b"img005.imgcut";
const LINEAR_FILTER: i32 = 0x2601;

pub fn load_scene_sheets(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.scene_img005_sheet = texture_cache_load(ctx, POPUP_PNG, POPUP_CUT, LINEAR_FILTER)?;

    Ok(())
}
