use crate::Fault;

use super::{AppContext, TextRenderer};

pub fn text_texture_cache(
    ctx: &mut AppContext,
) -> Result<&mut (dyn TextRenderer + 'static), Fault> {
    ctx.text_renderer().ok_or(Fault::HostMissing {
        site: "text_texture_cache",
    })
}
