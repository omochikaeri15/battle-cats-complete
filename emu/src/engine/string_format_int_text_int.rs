use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_int_text_int(ctx: &mut AppContext, pattern: &[u8], first: i32, text: &[u8], second: i32) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing { site: "string_format_int_text_int" })?
        .format_args(pattern, &[FormatArg::Int(first), FormatArg::Text(text), FormatArg::Int(second)]))
}
