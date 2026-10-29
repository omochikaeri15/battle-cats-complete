use super::AppContext;

pub fn web_popup_pending(ctx: &mut AppContext, kind: i32) -> bool {
    if !ctx.web_popup_pending.contains_key(&kind) {
        return false;
    }

    *ctx.web_popup_pending.entry(kind).or_insert(false)
}
