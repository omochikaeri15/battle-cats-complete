use std::collections::BTreeMap;

use crate::{Fault, operation};

use super::{
    AppContext, get_built_deck_rows, get_built_deck_stage_key, get_current_stage_id,
    get_deploy_cost, get_special_rule_params, get_unit_form, get_unit_rarity, read_flag,
};

pub fn unit_meets_restriction(
    ctx: &mut AppContext,
    slot: i32,
    in_battle: u8,
) -> Result<bool, Fault> {
    let stage_id = get_current_stage_id(ctx)?;
    let preset =
        (ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64) * AppContext::DECK_STRIDE as i64;
    let mut pair = [0u8; 8];

    pair[..4].copy_from_slice(
        &ctx.block_at::<4>(
            (preset + (slot as i64) * 4 + AppContext::DECK_PRESETS as i64) as usize,
        )?,
    );
    pair[4..].copy_from_slice(&ctx.block_at::<4>(
        (preset + (AppContext::DECK_PRESETS + AppContext::DECK_KEY) as i64) as usize,
    )?);

    let mut unit_id = (operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32)
        .wrapping_add(-2);
    let mut form = get_unit_form(ctx, unit_id)?;

    if ctx.i32_at(AppContext::SCENE_0X64_PAGE)? != 3 {
        if in_battle != 0 {
            pair[..4].copy_from_slice(
                &ctx.block_at::<4>(((slot as i64) * 4 + AppContext::BATTLE_DECK as i64) as usize)?,
            );
            pair[4..].copy_from_slice(
                &ctx.block_at::<4>(AppContext::BATTLE_DECK + AppContext::DECK_KEY)?,
            );
            unit_id = (operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32)
                .wrapping_add(-2);
            form =
                ctx.i32_at(((slot as i64) * 4 + AppContext::BUTTON_UNIT_FORMS as i64) as usize)?;
        } else if ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 {
            let stage_key = get_built_deck_stage_key(ctx)?;
            let rows = get_built_deck_rows(ctx, stage_key)?;
            let row = rows
                .get(slot as i64 as usize)
                .ok_or(Fault::index_out_of_range(slot as i64, 10))?;

            unit_id = (row.0 as i32).wrapping_add(-2);
            form = row.1 as i32;
        }
    }

    if unit_id < 0 {
        return Ok(false);
    }

    'record: {
        let Some(record) = ctx.stage_restrictions.get(&stage_id) else {
            break 'record;
        };
        let (rarity_mask, rows, min_cost, max_cost, group_id) = (
            record.rarity_mask,
            record.rows,
            record.min_cost,
            record.max_cost,
            record.group_id,
        );
        let rarity = get_unit_rarity(ctx, unit_id)?;

        if rarity_mask != 0 && (rarity_mask as u32 >> (rarity as u32 & 0x1f)) & 1 == 0 {
            return Ok(false);
        }

        if in_battle != 0 && (if rows != 1 { 10 } else { 5 }) <= slot {
            return Ok(false);
        }

        let mut cost = 0i32;

        if read_flag(ctx, AppContext::faction_flags(0))? & 1 != 0 {
            cost = operation::div_100(get_deploy_cost(ctx, unit_id, form, 1, -1)?);
        }

        if min_cost == 0 || max_cost == 0 {
            if min_cost != 0 {
                if cost < min_cost {
                    return Ok(false);
                }
            } else if max_cost != 0 && cost > max_cost {
                return Ok(false);
            }
        } else if min_cost == max_cost {
            if cost != min_cost {
                return Ok(false);
            }
        } else if min_cost >= max_cost {
            if cost < min_cost && cost > max_cost {
                return Ok(false);
            }
        } else if cost < min_cost || cost > max_cost {
            return Ok(false);
        }

        let Some(group) = ctx.chara_groups.get(&group_id) else {
            break 'record;
        };

        match group.kind {
            2 if group.units.contains(&unit_id) => return Ok(false),
            0 | 1 => {
                if group.kind == 1 && in_battle != 0 {
                    break 'record;
                }

                if !group.units.contains(&unit_id) {
                    return Ok(false);
                }
            }
            _ => {}
        }
    }

    let Some(params) =
        get_special_rule_params(ctx, &ctx.special_rules, operation::div_1000(stage_id), 2)?
    else {
        return Ok(true);
    };
    let mut counts: BTreeMap<i32, i32> = BTreeMap::new();

    if slot > 0 {
        let mut scan = 0i64;

        while scan != slot as u32 as i64 {
            let preset = (ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64)
                * AppContext::DECK_STRIDE as i64;

            pair[..4].copy_from_slice(
                &ctx.block_at::<4>((preset + scan * 4 + AppContext::DECK_PRESETS as i64) as usize)?,
            );
            pair[4..].copy_from_slice(&ctx.block_at::<4>(
                (preset + (AppContext::DECK_PRESETS + AppContext::DECK_KEY) as i64) as usize,
            )?);

            if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                <= 1
            {
                break;
            }

            let preset = (ctx.i32_at(AppContext::SELECTED_DECK_PRESET)? as i64)
                * AppContext::DECK_STRIDE as i64;

            pair[..4].copy_from_slice(
                &ctx.block_at::<4>((preset + scan * 4 + AppContext::DECK_PRESETS as i64) as usize)?,
            );
            pair[4..].copy_from_slice(&ctx.block_at::<4>(
                (preset + (AppContext::DECK_PRESETS + AppContext::DECK_KEY) as i64) as usize,
            )?);

            let listed = (operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32)
                .wrapping_add(-2);
            let rarity = get_unit_rarity(ctx, listed)?;
            let count = counts.entry(rarity).or_insert(0);

            *count = count.wrapping_add(1);
            scan += 1;
        }
    }

    let rarity = get_unit_rarity(ctx, unit_id)?;
    let have = *counts.entry(rarity).or_insert(0);
    let cap = *params
        .get(rarity as i64 as usize)
        .ok_or(Fault::index_out_of_range(rarity as i64, params.len() as i64))?;

    Ok(have < cap)
}
