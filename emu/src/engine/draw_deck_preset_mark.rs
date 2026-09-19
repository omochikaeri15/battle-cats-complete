use crate::{operation, Fault};

use super::{draw_context, draw_cut_scaled, get_map_type, AppContext};

const SITE: &str = "draw_deck_preset_mark";

pub fn draw_deck_preset_mark(ctx: &mut AppContext, slot: i32, x: i32, y: i32, width: i32, height: i32) -> Result<(), Fault> {
    if get_map_type(ctx, 0)? != -11 {
        return Ok(());
    }

    let key = ctx.block_at::<4>(AppContext::BATTLE_DECK_KEY)?;
    let cell = ctx.block_at::<4>(AppContext::BATTLE_DECK.wrapping_add(((slot as i64) * 4) as usize))?;
    let current = i32::from_le_bytes([key[0] ^ cell[0], key[1] ^ cell[1], key[2] ^ cell[2], key[3] ^ cell[3]]);
    let base = ((ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64) * AppContext::DECK_PRESET_STRIDE as i64) as usize;
    let key = ctx.block_at::<4>(base.wrapping_add(AppContext::DECK_PRESET_KEY))?;
    let cell = ctx.block_at::<4>(base.wrapping_add(AppContext::DECK_PRESETS).wrapping_add(((slot as i64) * 4) as usize))?;
    let saved = i32::from_le_bytes([key[0] ^ cell[0], key[1] ^ cell[1], key[2] ^ cell[2], key[3] ^ cell[3]]);

    if current == saved {
        return Ok(());
    }

    let sheet = ctx.img002_sheet.clone();
    let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, width, height, 0x2d);

    let flash = ctx.i32_at(AppContext::DEPLOY_FULL_FLASH)?;

    if flash as u32 > 0x31 {
        return Ok(());
    }

    let lit = (flash.wrapping_add(-0xf) as u32) < 0xa && flash & 2 == 0;
    let cut = 0x2ei32 | i32::from(lit);
    let size = if flash < 0x2e {
        0x6e
    } else {
        let step = flash.wrapping_add(-0x2e);

        ((0x6ei32.wrapping_mul(0x10i32.wrapping_sub(step.wrapping_mul(step)))) as u32 >> 4) as i32
    };
    let x = x.wrapping_add(operation::div_2(width)).wrapping_sub((size as u32 >> 1) as i32);

    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, x, y, size, height, cut);

    Ok(())
}
