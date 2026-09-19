use crate::Fault;

use super::{get_max_money, obfuscate_value, AppContext};

pub fn set_money(ctx: &mut AppContext, wallet: usize, amount: i32) -> Result<(), Fault> {
    let mut cell = [0u8; 8];

    if amount < 0 {
        cell[..4].copy_from_slice(&0i32.to_le_bytes());
    } else if get_max_money(ctx, wallet)? >= amount {
        cell[..4].copy_from_slice(&amount.to_le_bytes());
    } else {
        cell[..4].copy_from_slice(&get_max_money(ctx, wallet)?.to_le_bytes());
    }

    obfuscate_value(&mut cell);

    ctx.set_block_at(wallet.wrapping_add(AppContext::WALLET_MONEY), cell)
}
