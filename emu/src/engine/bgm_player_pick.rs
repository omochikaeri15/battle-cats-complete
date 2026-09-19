use crate::Fault;

use super::AppContext;

pub fn bgm_player_pick(ctx: &mut AppContext) -> Result<i32, Fault> {
    if ctx.i32_at(AppContext::BGM_SWITCH_FRAME)? != ctx.i32_at(AppContext::BGM_SWITCH_FRAMES)? {
        return Ok(-1);
    }

    let boss_phase = ctx.i32_at(AppContext::BGM_BOSS_PHASE)?;
    let row = (ctx.i32_at(AppContext::STAGE_MUSIC_ROW)? as i64)
        .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE as i64) as usize;
    let row = AppContext::MAP_STAGE_ROWS.wrapping_add(row);

    let key = ctx.block_at::<4>(row.wrapping_add(0xb8))?;
    let music = ctx.block_at::<4>(row.wrapping_add(8))?;
    let boss_music = ctx.block_at::<4>(row.wrapping_add(0x10))?;

    let normal = (music[0] ^ key[0]) as u32
        | ((music[1] ^ key[1]) as u32) << 8
        | ((music[2] ^ key[2]) as u32) << 0x10
        | ((music[3] ^ key[3]) as u32) << 0x18;
    let boss = (boss_music[0] ^ key[0]) as u32
        | ((boss_music[1] ^ key[1]) as u32) << 8
        | ((boss_music[2] ^ key[2]) as u32) << 0x10
        | ((boss_music[3] ^ key[3]) as u32) << 0x18;

    if normal == boss && boss_phase != 0 {
        return Ok(-1);
    }

    let picked = if boss_phase == 0 { normal } else { boss };

    ctx.set_block_at::<1>(AppContext::BGM_PENDING, [0])?;
    ctx.set_i32_at(AppContext::BGM_DELAY_FRAME, 0)?;
    ctx.set_i32_at(AppContext::BGM_DELAY, 0)?;

    Ok(picked as i32)
}
