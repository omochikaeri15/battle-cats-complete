use crate::Fault;

use super::{AppContext, FormatArg, analytics_send_event, get_item_count};

pub fn analytics_nekokan_use(
    ctx: &mut AppContext,
    at: i32,
    amount: i32,
    _flag: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    let mut list = Vec::with_capacity(params.len() + 3);

    list.extend_from_slice(params);
    list.push((b"paid".as_slice(), FormatArg::Int(0)));
    list.push((b"free".as_slice(), FormatArg::Int(get_item_count(ctx, 0x16)?)));
    list.push((b"total".as_slice(), FormatArg::Int(get_item_count(ctx, 0x16)?)));

    analytics_send_event(ctx, at, b"nekokan_use", amount, &list)
}
