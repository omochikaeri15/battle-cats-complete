use crate::Fault;

use super::AppContext;

pub fn set_deck_cooldown(
    ctx: &mut AppContext,
    wallet: usize,
    button: i32,
    value: i32,
    set_max: i32,
) -> Result<(), Fault> {
    let bytes = value.to_le_bytes();
    let key = ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_KEY))?;
    let at = ((button as i64) * 4) as usize;

    ctx.set_block_at(
        wallet
            .wrapping_add(AppContext::WALLET_COOLDOWNS)
            .wrapping_add(at),
        [
            key[0] ^ bytes[0],
            key[1] ^ bytes[1],
            key[2] ^ bytes[2],
            key[3] ^ bytes[3],
        ],
    )?;

    if set_max != 0 {
        let max_key =
            ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_MAX_KEY))?;

        ctx.set_block_at(
            wallet
                .wrapping_add(AppContext::WALLET_COOLDOWN_MAXES)
                .wrapping_add(at),
            [
                max_key[0] ^ bytes[0],
                bytes[1] ^ max_key[1],
                bytes[2] ^ max_key[2],
                bytes[3] ^ max_key[3],
            ],
        )?;
    }

    Ok(())
}
