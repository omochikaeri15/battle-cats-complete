use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_rank_comment(
    ctx: &mut AppContext,
    pattern: &[u8],
    rank: i32,
    name: &[u8],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::host_missing())?
        .format_args(pattern, &[FormatArg::Int(rank), FormatArg::Text(name)]))
}
