use crate::Fault;

use super::{AppContext, FormatArg};

pub fn format_localized(
    ctx: &mut AppContext,
    pattern: &[u8],
    first: &[u8],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing {
            site: "format_localized",
        })?
        .format_args(pattern, &[FormatArg::Text(first)]))
}
