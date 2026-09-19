use crate::Fault;

use super::{text_texture_cache, AppContext};

pub fn get_text_width(ctx: &mut AppContext, text: &[u8], size: i32) -> Result<i32, Fault> {
    Ok(text_texture_cache(ctx)?.text_width(text, size))
}
