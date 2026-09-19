use std::rc::Rc;

use crate::Fault;

use super::{AppContext, Imgcut};

pub fn option_window_init(
    ctx: &mut AppContext,
    sheet: Option<Rc<Imgcut>>,
    kind: i32,
) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::OPTION_WINDOW, [0])?;
    ctx.set_i32_at(AppContext::OPTION_WINDOW_KIND, kind)?;
    ctx.option_window_sheet = sheet;

    Ok(())
}
