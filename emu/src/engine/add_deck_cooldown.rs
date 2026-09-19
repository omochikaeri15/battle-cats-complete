use crate::Fault;

use super::AppContext;

pub fn add_deck_cooldown(ctx: &mut AppContext, wallet: usize, button: i32, delta: i32) -> Result<(), Fault> {
    let at = wallet.wrapping_add(AppContext::WALLET_COOLDOWNS).wrapping_add(((button as i64) * 4) as usize);
    let key = ctx.block_at::<4>(wallet.wrapping_add(AppContext::WALLET_COOLDOWN_KEY))?;
    let cell = ctx.block_at::<4>(at)?;
    let low = ((cell[0] ^ key[0]) as i32).wrapping_add(delta);
    let second = (((key[1] ^ cell[1]) as u32) << 8).wrapping_add(low as u32);
    let third = (((key[2] ^ cell[2]) as u32) << 0x10).wrapping_add(second);
    let high = ((third >> 0x18) as u8).wrapping_add(key[3] ^ cell[3]);

    ctx.set_block_at(at, [low as u8 ^ key[0], (second >> 8) as u8 ^ key[1], (third >> 0x10) as u8 ^ key[2], high ^ key[3]])
}
