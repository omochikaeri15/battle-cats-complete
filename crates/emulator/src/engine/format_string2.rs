use crate::Fault;

use super::AppContext;

pub fn format_string2(
    ctx: &mut AppContext,
    pattern: &[u8],
    first: &[u8],
    second: &[u8],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::host_missing())?
        .format(pattern, &[first, second]))
}
