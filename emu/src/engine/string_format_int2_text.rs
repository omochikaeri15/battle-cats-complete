use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_int2_text(ctx: &mut AppContext, pattern: &[u8], first: i32, second: i32, text: &[u8]) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing { site: "string_format_int2_text" })?
        .format_args(pattern, &[FormatArg::Int(first), FormatArg::Int(second), FormatArg::Text(text)]))
}
