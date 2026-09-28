use crate::Fault;

use super::{AppContext, FormatArg, analytics_send_event, get_item_count};

pub fn analytics_nekokan_get(
    ctx: &mut AppContext,
    at: i32,
    amount: i32,
    flag: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    let mut list = Vec::with_capacity(params.len() + 4);

    list.extend_from_slice(params);
    list.push((b"paid".as_slice(), FormatArg::Int(0)));
    list.push((b"free".as_slice(), FormatArg::Int(get_item_count(ctx, 0x16)?)));
    list.push((b"total".as_slice(), FormatArg::Int(get_item_count(ctx, 0x16)?)));

    if flag == 0 {
        list.push((b"price".as_slice(), FormatArg::Int(0)));
    }

    analytics_send_event(ctx, at, b"nekokan_get", amount, &list)
}
