use crate::Fault;

use super::AppContext;

pub fn set_deck_cooldown_max(
    ctx: &mut AppContext,
    wallet: usize,
    button: i32,
    value: i32,
) -> Result<(), Fault> {
    let key = ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_MAX_KEY))?;
    let bytes = value.to_le_bytes();

    ctx.set_block_at(
        wallet
            .wrapping_add(AppContext::WALLET_COOLDOWN_MAXES)
            .wrapping_add(((button as i64) * 4) as usize),
        [
            key[0] ^ bytes[0],
            key[1] ^ bytes[1],
            key[2] ^ bytes[2],
            key[3] ^ bytes[3],
        ],
    )
}
