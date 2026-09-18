use crate::Fault;

use super::{get_entity_button, orb_deploy_condition, AppContext};

pub fn orb_deploy_condition_slot(ctx: &AppContext, wallet: usize, faction: i32, slot: i32) -> Result<bool, Fault> {
    let button = get_entity_button(ctx, faction, slot)?;

    orb_deploy_condition(ctx, wallet, faction, button)
}
