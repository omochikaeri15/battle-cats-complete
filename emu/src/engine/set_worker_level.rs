use crate::Fault;

use super::{obfuscate_value, AppContext};

pub fn set_worker_level(ctx: &mut AppContext, wallet: usize, level: i32) -> Result<(), Fault> {
    let mut cell = [0u8; 8];

    if level < 0 {
        cell[..4].copy_from_slice(&0i32.to_le_bytes());
    } else if (level as u32) < 8 {
        cell[..4].copy_from_slice(&level.to_le_bytes());
    } else {
        cell[..4].copy_from_slice(&7i32.to_le_bytes());
    }

    obfuscate_value(&mut cell);

    ctx.set_block_at(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL), cell)
}
