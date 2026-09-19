use crate::Fault;

use super::{AppContext, get_base_upgrade};

pub fn get_worker_upgrade_cost(ctx: &mut AppContext, wallet: usize) -> Result<i32, Fault> {
    let tier = get_base_upgrade(ctx, 4)?;
    let flat_upgrade = get_base_upgrade(ctx, 4)?;
    let scaled_upgrade = get_base_upgrade(ctx, 4)?;
    let cell = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL))?;
    let worker_level = ((cell[7] ^ cell[0]) as u32
        | ((cell[6] ^ cell[1]) as u32) << 8
        | ((cell[5] ^ cell[2]) as u32) << 0x10
        | ((cell[4] ^ cell[3]) as u32) << 0x18) as i32;
    let step = if tier >= 7 { 0x7d0 } else { 0x3e8 };
    let offset = if tier >= 7 { -0x7d0 } else { 0xfa0 };
    let flat = flat_upgrade.wrapping_mul(step).wrapping_add(offset);
    let scaled = scaled_upgrade
        .wrapping_mul(step)
        .wrapping_add(offset)
        .wrapping_mul(worker_level);

    Ok(scaled.wrapping_add(flat))
}
