use crate::Fault;

use super::{AppContext, FormatArg};

#[allow(clippy::too_many_arguments)]
pub fn string_format_boss_hp_line(ctx: &mut AppContext, pattern: &[u8], line: &[u8], open: &[u8], label: &[u8], percent: i32, close: &[u8]) -> Result<Vec<u8>, Fault> {
    Ok(ctx.text_renderer().ok_or(Fault::HostMissing { site: "string_format_boss_hp_line" })?.format_args(
        pattern,
        &[FormatArg::Text(line), FormatArg::Text(open), FormatArg::Text(label), FormatArg::Int(percent), FormatArg::Text(close)],
    ))
}
