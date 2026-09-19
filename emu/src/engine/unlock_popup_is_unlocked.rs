use super::AppContext;

pub fn unlock_popup_is_unlocked(ctx: &mut AppContext, id: i32) -> bool {
    if !ctx.unlock_popups.contains_key(&id) {
        return false;
    }

    *ctx.unlock_popups.entry(id).or_insert(false)
}
