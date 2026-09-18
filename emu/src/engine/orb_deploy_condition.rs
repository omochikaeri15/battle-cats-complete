use crate::Fault;

use super::{get_button_unit_form, get_button_unit_id, get_setting, get_unit_rarity, AppContext};

pub fn orb_deploy_condition(ctx: &AppContext, wallet: usize, faction: i32, slot: i32) -> Result<bool, Fault> {
    if faction != 0 {
        return Ok(false);
    }

    if get_button_unit_form(ctx, 0, slot)? < 2 {
        return Ok(false);
    }

    let rarity = get_unit_rarity(ctx, get_button_unit_id(ctx, 0, slot)?)?;
    let mut key = rarity.to_string().into_bytes();

    key.splice(0..0, *b"battle_af_condition_production_");

    let kind = get_setting(&ctx.settings, b"battle_af_condition_production_type", 0)?;
    let deploys = ctx
        .i32_at(wallet.wrapping_add(((slot as i64) << 2) as usize).wrapping_add(AppContext::WALLET_DEPLOY_COUNTS))?
        .wrapping_add(1);

    if kind == 0 {
        let every = get_setting(&ctx.settings, &key, 2)?;

        if every == 0 {
            return Err(Fault::DivideByZero { site: "orb_deploy_condition" });
        }

        if deploys == i32::MIN && every == -1 {
            return Err(Fault::DivideOverflow { site: "orb_deploy_condition" });
        }

        return Ok(deploys % every == 0);
    }

    Ok(deploys == get_setting(&ctx.settings, &key, 2)?)
}
