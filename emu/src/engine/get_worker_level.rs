use crate::Fault;

use super::AppContext;

pub fn get_worker_level(ctx: &AppContext, wallet: usize) -> Result<i32, Fault> {
    let cell = ctx.block_at::<8>(wallet.wrapping_add(AppContext::WALLET_WORKER_LEVEL))?;
    let low = (cell[7] ^ cell[0]) as u32;
    let second = ((cell[6] ^ cell[1]) as u32) << 8;
    let third = ((cell[5] ^ cell[2]) as u32) << 0x10;
    let high = ((cell[4] ^ cell[3]) as u32) << 0x18;

    Ok((low | second | third | high) as i32)
}
