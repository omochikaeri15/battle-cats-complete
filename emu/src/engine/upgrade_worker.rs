use crate::Fault;

use super::{AppContext, obfuscate_value};

pub fn upgrade_worker(ctx: &mut AppContext, wallet: usize) -> Result<(), Fault> {
    let mut cell = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL))?;
    let low = (cell[7] ^ cell[0]) as u32;
    let second = ((cell[6] ^ cell[1]) as u32) << 8;
    let third = ((cell[5] ^ cell[2]) as u32) << 0x10;
    let high = ((cell[4] ^ cell[3]) as u32) << 0x18;
    let next = (low | high | second).wrapping_add(third).wrapping_add(1);

    cell[..4].copy_from_slice(&next.to_le_bytes());
    obfuscate_value(&mut cell);

    ctx.set_block_at(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL), cell)
}
