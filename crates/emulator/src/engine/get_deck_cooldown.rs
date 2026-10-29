use crate::Fault;

use super::AppContext;

pub fn get_deck_cooldown(ctx: &AppContext, wallet: usize, button: i32) -> Result<i32, Fault> {
    let key = ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_KEY))?;
    let cell = ctx.block_at::<4>(
        wallet
            .wrapping_add(AppContext::WALLET_COOLDOWNS)
            .wrapping_add(((button as i64) * 4) as usize),
    )?;

    Ok(i32::from_le_bytes([
        key[0] ^ cell[0],
        key[1] ^ cell[1],
        key[2] ^ cell[2],
        key[3] ^ cell[3],
    ]))
}
