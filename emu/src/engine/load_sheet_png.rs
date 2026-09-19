use crate::Fault;

use super::{AppContext, Imgcut};

pub fn load_sheet_png(ctx: &mut AppContext, sheet: &mut Imgcut) -> Result<bool, Fault> {
    let Some(image) = ctx.assets().ok_or(Fault::HostMissing { site: "load_sheet_png" })?.load_png(&sheet.png) else {
        return Ok(false);
    };

    sheet.width = image.width;
    sheet.height = image.height;
    sheet.whole = image.whole;

    Ok(true)
}
