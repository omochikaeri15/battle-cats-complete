use crate::Fault;

use super::{AppContext, Imgcut, imgcut_load, texture_upload};

pub fn texture_load(
    ctx: &mut AppContext,
    sheet: &mut Imgcut,
    png: &[u8],
    cut: &[u8],
    flag: i32,
) -> Result<bool, Fault> {
    let loaded = imgcut_load(ctx, sheet, png, cut, flag)?;

    texture_upload(ctx, sheet)?;

    Ok(loaded)
}
