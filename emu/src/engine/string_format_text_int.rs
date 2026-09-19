use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_text_int(ctx: &mut AppContext, pattern: &[u8], text: &[u8], value: i32) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing { site: "string_format_text_int" })?
        .format_args(pattern, &[FormatArg::Text(text), FormatArg::Int(value)]))
}
