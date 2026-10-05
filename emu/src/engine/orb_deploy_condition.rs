use crate::Fault;

use super::{AppContext, deploy_count_condition, get_button_unit_form};

pub fn orb_deploy_condition(
    ctx: &AppContext,
    wallet: usize,
    faction: i32,
    slot: i32,
) -> Result<bool, Fault> {
    if faction == 0 && get_button_unit_form(ctx, 0, slot)? >= 2 {
        return deploy_count_condition(ctx, wallet, 0, slot);
    }

    Ok(false)
}
