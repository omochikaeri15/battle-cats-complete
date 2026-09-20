use crate::Fault;

use super::{AppContext, FormatArg};

pub fn string_format_boss_hp(
    ctx: &mut AppContext,
    pattern: &[u8],
    label: &[u8],
    percent: i32,
    suffix: &[u8],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::host_missing())?
        .format_args(
            pattern,
            &[
                FormatArg::Text(label),
                FormatArg::Int(percent),
                FormatArg::Text(suffix),
            ],
        ))
}
