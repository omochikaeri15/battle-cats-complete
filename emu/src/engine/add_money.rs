use crate::Fault;

use super::{get_max_money, obfuscate_value, AppContext};

pub fn add_money(ctx: &mut AppContext, wallet: usize, amount: i32) -> Result<(), Fault> {
    let held = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_MONEY))?;
    let low = (held[7] ^ held[0]) as u32;
    let second = ((held[6] ^ held[1]) as u32) << 8;
    let third = ((held[5] ^ held[2]) as u32) << 0x10;
    let high = ((held[4] ^ held[3]) as u32) << 0x18;
    let total = (low as i32).wrapping_add(amount).wrapping_add(second as i32).wrapping_add(third as i32).wrapping_add(high as i32);
    let mut cell = [0u8; 8];

    if total < 0 {
        cell[..4].copy_from_slice(&0i32.to_le_bytes());
    } else if get_max_money(ctx, wallet)? >= total {
        cell[..4].copy_from_slice(&total.to_le_bytes());
    } else {
        cell[..4].copy_from_slice(&get_max_money(ctx, wallet)?.to_le_bytes());
    }

    obfuscate_value(&mut cell);

    ctx.set_block_at(wallet.wrapping_add(AppContext::WALLET_MONEY), cell)
}
