use crate::{Fault, operation};

use super::{AppContext, get_cannon_unit_id, get_scene_id, read_flag, stat_conjure_unit_id};

pub fn get_button_unit_row(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    let scene = get_scene_id(ctx)?;

    if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 2 != 0 {
            return ctx
                .i32_at(((slot as i64) * 4 + AppContext::FACTION_1_BUTTON_ROWS as i64) as usize);
        }

        if read_flag(ctx, AppContext::faction_flags(1))? & 1 == 0 {
            return Ok(-1);
        }

        let value =
            ctx.block_at::<4>(((slot as i64) * 4 + AppContext::FACTION_1_DECK as i64) as usize)?;
        let key = ctx.block_at::<4>(AppContext::FACTION_1_DECK + AppContext::DECK_KEY)?;
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(&value);
        pair[4..].copy_from_slice(&key);

        return Ok(
            operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange {
                site: "get_button_unit_row",
                index: 0,
                limit: 1,
            })? as i32,
        );
    }

    if faction != 0 {
        return Ok(-1);
    }

    if slot == 0xa {
        return Ok(get_cannon_unit_id(ctx, 0)?.wrapping_add(2));
    }

    if slot > 0xa {
        let button = slot.wrapping_add(-0xb);
        let mut form = 0;

        if get_button_unit_row(ctx, 0, button)? == -1 {
            return Ok(-1);
        }

        let unit_id = get_button_unit_row(ctx, 0, button)?.wrapping_add(-2);

        if button as u32 <= 9 {
            form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as u32 as usize) * 4)?;
        }

        let conjured = stat_conjure_unit_id(ctx, 0, unit_id, form)?;

        return Ok(if conjured >= 0 {
            conjured.wrapping_add(2)
        } else {
            -1
        });
    }

    if scene == 0x12c {
        let value =
            ctx.block_at::<4>(((slot as i64) * 4 + AppContext::BATTLE_DECK as i64) as usize)?;
        let key = ctx.block_at::<4>(AppContext::BATTLE_DECK + AppContext::DECK_KEY)?;
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(&value);
        pair[4..].copy_from_slice(&key);

        return Ok(
            operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange {
                site: "get_button_unit_row",
                index: 0,
                limit: 1,
            })? as i32,
        );
    }

    let preset =
        (ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64) * AppContext::DECK_STRIDE as i64;
    let value =
        ctx.block_at::<4>((preset + (slot as i64) * 4 + AppContext::DECK_PRESETS as i64) as usize)?;
    let key = ctx.block_at::<4>(
        (preset + (AppContext::DECK_PRESETS + AppContext::DECK_KEY) as i64) as usize,
    )?;
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(&value);
    pair[4..].copy_from_slice(&key);

    Ok(
        operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange {
            site: "get_button_unit_row",
            index: 0,
            limit: 1,
        })? as i32,
    )
}
