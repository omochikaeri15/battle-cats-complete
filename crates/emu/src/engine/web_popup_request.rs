use super::AppContext;

pub fn web_popup_request(ctx: &mut AppContext, kind: i32) {
    *ctx.web_popup_pending.entry(kind).or_insert(false) = true;
}
