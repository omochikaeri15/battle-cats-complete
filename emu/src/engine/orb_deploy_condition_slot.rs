use crate::Fault;

use super::{AppContext, deploy_count_condition, get_button_unit_form, get_entity_button};

pub fn orb_deploy_condition_slot(
    ctx: &AppContext,
    wallet: usize,
    faction: i32,
    slot: i32,
) -> Result<bool, Fault> {
    let button = get_entity_button(ctx, faction, slot)?;

    if faction == 0 && get_button_unit_form(ctx, 0, button)? >= 2 {
        return deploy_count_condition(ctx, wallet, 0, button);
    }

    Ok(false)
}
