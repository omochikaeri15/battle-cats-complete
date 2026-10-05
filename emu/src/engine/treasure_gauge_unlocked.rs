use super::{AppContext, unlock_popup_is_unlocked};

pub fn treasure_gauge_unlocked(ctx: &mut AppContext) -> bool {
    unlock_popup_is_unlocked(ctx, 0xa1)
}
