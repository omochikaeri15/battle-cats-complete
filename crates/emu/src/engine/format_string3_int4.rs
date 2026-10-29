use crate::Fault;

use super::{AppContext, FormatArg};

pub fn format_string3_int4(
    ctx: &mut AppContext,
    pattern: &[u8],
    first: &[u8],
    second: &[u8],
    third: &[u8],
    numbers: [i32; 4],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::host_missing())?
        .format_args(
            pattern,
            &[
                FormatArg::Text(first),
                FormatArg::Text(second),
                FormatArg::Text(third),
                FormatArg::Int(numbers[0]),
                FormatArg::Int(numbers[1]),
                FormatArg::Int(numbers[2]),
                FormatArg::Int(numbers[3]),
            ],
        ))
}
