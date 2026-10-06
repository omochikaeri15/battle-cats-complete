use crate::Fault;

use super::{
    AppContext, deploy_count_condition, get_button_unit_form, get_button_unit_id, get_equipped_orb,
    get_orb_def, orb_ability_flag,
};

pub fn orb_icon_visible(
    ctx: &mut AppContext,
    wallet: usize,
    faction: i32,
    slot: i32,
    orb_slot: i32,
    ignore_cooldown: u8,
) -> Result<bool, Fault> {
    let unit = get_button_unit_id(ctx, faction, slot)?;
    let orb = get_equipped_orb(ctx, unit, orb_slot)?;

    if orb == -1 {
        return Ok(false);
    }

    let ability = get_orb_def(&ctx.orb_store, orb)?.ability;

    if !orb_ability_flag(&mut ctx.orb_store, ability) {
        return Ok(true);
    }

    if faction != 0 || get_button_unit_form(ctx, 0, slot)? < 2 {
        return Ok(false);
    }

    if !deploy_count_condition(ctx, wallet, 0, slot)? {
        return Ok(false);
    }

    let key = ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_KEY))?;
    let cell = ctx.block_at::<4>(
        wallet
            .wrapping_add(AppContext::WALLET_COOLDOWNS)
            .wrapping_add(((slot as i64) * 4) as usize),
    )?;
    let idle =
        ((key[0] ^ cell[0]) | (key[1] ^ cell[1]) | (key[2] ^ cell[2]) | (key[3] ^ cell[3])) == 0;

    Ok(((ignore_cooldown ^ 1) | u8::from(idle)) != 0)
}
