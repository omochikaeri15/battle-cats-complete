use crate::Fault;

use super::{AppContext, FormatArg, analytics_send_event};

pub fn analytics_stamina_get(
    ctx: &mut AppContext,
    at: i32,
    amount: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    analytics_send_event(ctx, at, b"stamina_get", amount, params)
}
