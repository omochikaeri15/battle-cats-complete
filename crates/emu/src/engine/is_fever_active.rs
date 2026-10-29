use crate::Fault;

use super::{AppContext, SpecialRuleStore, get_scene_id};

pub fn is_fever_active(ctx: &AppContext, store: &SpecialRuleStore) -> Result<bool, Fault> {
    let battle = get_scene_id(ctx)? == 0x12c;

    Ok(store.fever_count > 0 && battle)
}
