use crate::Fault;

use super::AppContext;

pub fn substitute_tokens(
    ctx: &mut AppContext,
    text: &[u8],
    tokens: &[(&[u8], &[u8])],
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .text_renderer()
        .ok_or(Fault::HostMissing {
            site: "substitute_tokens",
        })?
        .substitute(text, tokens))
}
