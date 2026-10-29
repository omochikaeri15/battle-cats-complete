use crate::Fault;

use super::{AppContext, get_button_unit_id, get_setting, get_unit_rarity};

pub fn deploy_count_condition(
    ctx: &AppContext,
    wallet: usize,
    faction: i32,
    slot: i32,
) -> Result<bool, Fault> {
    let rarity = get_unit_rarity(ctx, get_button_unit_id(ctx, faction, slot)?)?;
    let mut key = rarity.to_string().into_bytes();

    key.splice(0..0, *b"battle_af_condition_production_");

    let kind = get_setting(&ctx.settings, b"battle_af_condition_production_type", 0)?;
    let deploys = ctx
        .i32_at(
            wallet
                .wrapping_add(((slot as i64) << 2) as usize)
                .wrapping_add(AppContext::WALLET_DEPLOY_COUNTS),
        )?
        .wrapping_add(1);

    if kind == 0 {
        let every = get_setting(&ctx.settings, &key, 2)?;

        if every == 0 {
            return Err(Fault::divide_by_zero());
        }

        if deploys == i32::MIN && every == -1 {
            return Err(Fault::divide_overflow());
        }

        return Ok(deploys % every == 0);
    }

    Ok(deploys == get_setting(&ctx.settings, &key, 2)?)
}
