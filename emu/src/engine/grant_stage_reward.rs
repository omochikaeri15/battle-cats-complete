use crate::Fault;

use super::{
    AppContext, FormatArg, UNIT_BUY, UNIT_BUY_STRIDE, add_resource, analytics_params,
    event_unit_slot_by_item, find_item_index, get_global_map_id, get_powerup, get_stage_index,
    get_crown_level, level_cell_add, level_cell_plus, mission_progress, orb_inventory_add,
    reward_unit_id, unit_buy_field, xor_row46_get,
};

pub fn grant_stage_reward(ctx: &mut AppContext, field: i32, first: u8) -> Result<i32, Fault> {
    let row = (AppContext::MAP_STAGE_ROWS as i64
        + (ctx.i32_at(AppContext::STAGE_ROW)? as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64)
        as usize;
    let index = field as i64 as usize;
    let item = xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32;
    let kind = find_item_index(ctx, item)?;
    let item = xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32;

    if kind != -1 {
        let kind = find_item_index(ctx, item)?;
        let amount =
            xor_row46_get(ctx.bytes_from(row)?, index + 1).ok_or(Fault::index_out_of_range(index as i64 + 1, 0x2e))? as i32;

        add_resource(ctx, kind, amount, 0)?;

        let item = xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32;

        if find_item_index(ctx, item)? == 0x16 {
            let map_id = get_global_map_id(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let crown = get_crown_level(ctx)?;
            let amount =
                xor_row46_get(ctx.bytes_from(row)?, index + 1).ok_or(Fault::index_out_of_range(index as i64 + 1, 0x2e))? as i32;

            analytics_params(
                ctx,
                0x98c17f + first as i32,
                amount,
                &[
                    (b"sec1_type", FormatArg::Text(b"MapID")),
                    (b"sec1_id", FormatArg::Int(map_id)),
                    (b"sec2_type", FormatArg::Text(b"StageIdx")),
                    (b"sec2_id", FormatArg::Int(stage)),
                    (b"ex_type", FormatArg::Text(b"StageLv")),
                    (b"ex_id", FormatArg::Int(crown)),
                ],
            )?;
        }

        return Ok(0);
    }

    if item < 0x3e8 {
        return Ok(0);
    }

    'owned: {
        let (slot, unit, bound, limit) = 'unit: {
            if item > ctx.drop_chara_max_1000 {
                if item == 0x3ea {
                    if ctx.i32_at(AppContext::REWARD_1002_OWNED)? == 0 {
                        ctx.set_i32_at(AppContext::REWARD_1002_OWNED, 1)?;
                        ctx.set_i32_at(AppContext::UNITS_UNLOCKED_FLAG, 1)?;

                        return Ok(2);
                    }

                    break 'owned;
                }

                if item as u32 >= 0x44c {
                    if item <= ctx.drop_chara_max_1100 {
                        let slot = event_unit_slot_by_item(&ctx.drop_chara_rows, item)?;
                        let value = xor_row46_get(ctx.bytes_from(row)?, index).ok_or(
                            Fault::index_out_of_range(index as i64, 0x2e),
                        )? as i32;
                        let unit = reward_unit_id(ctx, value)?;

                        break 'unit (slot, unit, 0x44c, ctx.drop_chara_max_1100);
                    }

                    if item as u32 >= 0x2710 {
                        if item as u32 > 0x752f {
                            let amount = xor_row46_get(ctx.bytes_from(row)?, index + 1).ok_or(
                                Fault::index_out_of_range(index as i64 + 1, 0x2e),
                            )? as i32;

                            orb_inventory_add(ctx, item.wrapping_sub(0x7530), amount);

                            return Ok(0);
                        }

                        for unit in 0..0x36cusize {
                            let buy = UNIT_BUY + unit * UNIT_BUY_STRIDE;

                            if unit_buy_field(ctx.bytes_from(buy)?, 0x17)? == item {
                                let owned = AppContext::REWARD_UNITS_OWNED + unit * 4;

                                if ctx.i32_at(owned)? != 0 {
                                    break 'owned;
                                }

                                ctx.set_i32_at(owned, 1)?;

                                return Ok(6);
                            }

                            if unit_buy_field(ctx.bytes_from(buy)?, 0x18)? == item {
                                let owned = AppContext::REWARD_FORMS_OWNED + unit * 4;

                                if ctx.i32_at(owned)? != 0 {
                                    break 'owned;
                                }

                                ctx.set_i32_at(owned, 1)?;

                                return Ok(6);
                            }
                        }
                    }
                }

                return Ok(0);
            }

            if item == 0x3ea {
                if ctx.i32_at(AppContext::REWARD_1002_OWNED)? == 0 {
                    ctx.set_i32_at(AppContext::REWARD_1002_OWNED, 1)?;
                    ctx.set_i32_at(AppContext::UNITS_UNLOCKED_FLAG, 1)?;

                    return Ok(2);
                }

                break 'owned;
            }

            let slot = event_unit_slot_by_item(&ctx.drop_chara_rows, item)?;
            let value =
                xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32;
            let unit = reward_unit_id(ctx, value)?;

            break 'unit (slot, unit, 0x3e8, ctx.drop_chara_max_1000);
        };
        let owned = ((slot as i64) * 4 + AppContext::EVENT_UNIT_OWNED as i64) as usize;

        if ctx.i32_at(owned)? == 0 {
            ctx.set_i32_at(owned, 1)?;
            ctx.set_i32_at(AppContext::UNITS_UNLOCKED_FLAG, 1)?;

            return Ok(2);
        }

        let value = xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32;

        if value >= bound
            && xor_row46_get(ctx.bytes_from(row)?, index).ok_or(Fault::index_out_of_range(index as i64, 0x2e))? as i32
                <= limit
        {
            break 'owned;
        }

        let cell = ((unit as i64) * 8 + AppContext::UNIT_LEVELS as i64) as usize;
        let buy = (((unit as i64) << 8) + UNIT_BUY as i64) as usize;

        if level_cell_plus(ctx, cell)? >= unit_buy_field(ctx.bytes_from(buy)?, 0x13)? {
            return Ok(5);
        }

        let result = 3
            + (level_cell_plus(ctx, cell)?
                >= unit_buy_field(ctx.bytes_from(buy)?, 0x13)?.wrapping_sub(1))
                as i32;

        level_cell_add(ctx, cell, 1)?;
        mission_progress(ctx, 0x11, 0, 1, 0, 0)?;

        return Ok(result);
    }

    if first == 0 && get_powerup(ctx, 1)? {
        ctx.set_block_at::<1>(AppContext::RANK_POPUP_SHOWN, [1])?;
    }

    Ok(1)
}
