use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_int(
    ctx: &mut AppContext,
    pattern: &[u8],
    value: i32,
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing {
            site: "string_format_int",
        })?
        .format_args(pattern, &[FormatArg::Int(value)]))
}
