use crate::Fault;

use super::{
    AppContext, UNIT_BUY, UNIT_BUY_STRIDE, find_item_index, format_string3, format_string3_int3,
    format_string3_int4, get_item_name, get_orb_def, level_cell_plus, level_cell_base, orb_name,
    query_localizable, reward_unit_id, std_string_from_cstr, substitute_tokens, unit_buy_field,
    xor_row46_get,
};

pub fn drop_popup_text(
    ctx: &mut AppContext,
    cell: i32,
    score: u8,
    mode: i32,
) -> Result<Vec<u8>, Fault> {
    let reward_name = if score != 0 {
        query_localizable(ctx, b"score_reward")
    } else {
        query_localizable(ctx, b"drop_reward")
    };

    if ctx.u8_at(AppContext::RANK_POPUP_SHOWN)? != 0 {
        let pattern = query_localizable(ctx, b"item_possession");
        let held = ctx.item_possession.entry(1).or_insert(0).to_string();
        let after = ctx
            .item_possession
            .entry(1)
            .or_insert(0)
            .wrapping_add(1)
            .to_string();
        let possession = substitute_tokens(
            ctx,
            &pattern,
            &[
                (b"itemNum1".as_slice(), held.as_bytes()),
                (b"itemNum2".as_slice(), after.as_bytes()),
            ],
        )?;
        let count = ctx.item_possession.entry(1).or_insert(0);

        *count = count.wrapping_add(1);

        let pattern = query_localizable(ctx, b"drop_popup_item");
        let reward = query_localizable(ctx, b"drop_reward");
        let name = get_item_name(ctx, 1);
        let amount = 1i32.to_string();

        return substitute_tokens(
            ctx,
            &pattern,
            &[
                (b"rewardName".as_slice(), reward.as_slice()),
                (b"itemName".as_slice(), name.as_slice()),
                (b"itemNum".as_slice(), amount.as_bytes()),
                (b"itemPossession".as_slice(), possession.as_slice()),
            ],
        );
    }

    let row = AppContext::MAP_STAGE_ROWS.wrapping_add(
        (ctx.i32_at(AppContext::STAGE_ROW)? as i64 as usize)
            .wrapping_mul(AppContext::MAP_STAGE_ROW_STRIDE),
    );
    let item_id = xor_row46_get(ctx.bytes_from(row)?, cell as i64 as usize)
        .ok_or(Fault::index_out_of_range(cell as i64, 0x2f))? as i32;
    let index = find_item_index(ctx, item_id)?;

    if index == -1 {
        if item_id < 0x3e8 {
            return Ok(Vec::new());
        }

        let unit = 'unit: {
            if item_id <= ctx.drop_chara_max_1000 {
                if item_id == 0x3ea {
                    break 'unit 0x1b;
                }

                break 'unit reward_unit_id(ctx, item_id)?;
            }

            if (item_id as u32) < 0x44c {
                return Ok(Vec::new());
            }

            if item_id <= ctx.drop_chara_max_1100 {
                break 'unit reward_unit_id(ctx, item_id)?;
            }

            if (item_id as u32) < 0x2710 {
                return Ok(Vec::new());
            }

            if item_id as u32 > 0x752f {
                if item_id as u32 > 0x9c3f {
                    return Ok(Vec::new());
                }

                let next = cell.wrapping_add(1) as i64 as usize;
                let pattern = query_localizable(ctx, &std_string_from_cstr(b"item_possession"));
                let held = *ctx.item_possession.entry(item_id).or_insert(0);
                let gained = xor_row46_get(ctx.bytes_from(row)?, next)
                    .ok_or(Fault::index_out_of_range(next as i64, 0x2f))?
                    as i32;
                let possession = substitute_tokens(
                    ctx,
                    &pattern,
                    &[
                        (b"itemNum1".as_slice(), held.to_string().as_bytes()),
                        (
                            b"itemNum2".as_slice(),
                            gained.wrapping_add(held).to_string().as_bytes(),
                        ),
                    ],
                )?;
                let count = ctx.item_possession.entry(item_id).or_insert(0);

                *count = count.wrapping_add(gained);

                let pattern = query_localizable(ctx, &std_string_from_cstr(b"equipment_drop"));
                let def = get_orb_def(&ctx.orb_store, item_id.wrapping_sub(0x7530))?.clone();
                let name = orb_name(ctx, &def)?;

                return substitute_tokens(
                    ctx,
                    &pattern,
                    &[
                        (b"rewardName".as_slice(), reward_name.as_slice()),
                        (b"equipmentName".as_slice(), name.as_slice()),
                        (b"equipmentNum".as_slice(), gained.to_string().as_bytes()),
                        (b"itemPossession".as_slice(), possession.as_slice()),
                    ],
                );
            }

            for unit in 0..0x36cusize {
                let buy = UNIT_BUY.wrapping_add(unit.wrapping_mul(UNIT_BUY_STRIDE));
                let key = if unit_buy_field(ctx.bytes_from(buy)?, 0x17)? == item_id {
                    b"drop_popup_chara_third_evolve".as_slice()
                } else if unit_buy_field(ctx.bytes_from(buy)?, 0x18)? == item_id {
                    b"drop_popup_chara_third_evolve2".as_slice()
                } else {
                    continue;
                };
                let pattern = query_localizable(ctx, &std_string_from_cstr(key));
                let mut rarity = unit_buy_field(ctx.bytes_from(buy)?, 0xd)?
                    .to_string()
                    .into_bytes();

                rarity.splice(0..0, b"drop_rarity_".iter().copied());

                let rarity = query_localizable(ctx, &rarity);
                let name = ctx
                    .cat_names
                    .get(unit)
                    .map_or_else(Vec::new, |forms| forms[0][0].clone());

                return format_string3(ctx, &pattern, &reward_name, &rarity, &name);
            }

            return Ok(Vec::new());
        };

        if mode.wrapping_sub(2) as u32 > 3 {
            return Ok(Vec::new());
        }

        let buy = UNIT_BUY.wrapping_add((unit as i64 as usize).wrapping_mul(UNIT_BUY_STRIDE));
        let levels = AppContext::UNIT_LEVELS.wrapping_add((unit as i64 as usize).wrapping_mul(8));
        let mut rarity = unit_buy_field(ctx.bytes_from(buy)?, 0xd)?
            .to_string()
            .into_bytes();

        rarity.splice(0..0, b"drop_rarity_".iter().copied());

        let name = ctx
            .cat_names
            .get(unit as i64 as usize)
            .map_or_else(Vec::new, |forms| forms[0][0].clone());

        return match mode.wrapping_sub(2) {
            0 => {
                let pattern =
                    query_localizable(ctx, &std_string_from_cstr(b"drop_popup_chara_first"));
                let rarity = query_localizable(ctx, &rarity);

                format_string3(ctx, &pattern, &reward_name, &rarity, &name)
            }
            1 | 2 => {
                let raised = mode.wrapping_sub(2) == 1;
                let base = level_cell_base(ctx, levels)?;
                let plus = level_cell_plus(ctx, levels)?;

                if level_cell_plus(ctx, levels)? != 1 {
                    let key = if raised {
                        b"drop_popup_chara_levelup2".as_slice()
                    } else {
                        b"drop_popup_chara_levelmax2".as_slice()
                    };
                    let pattern = query_localizable(ctx, &std_string_from_cstr(key));
                    let rarity = query_localizable(ctx, &rarity);

                    format_string3_int4(
                        ctx,
                        &pattern,
                        &reward_name,
                        &rarity,
                        &name,
                        [
                            base.wrapping_add(1),
                            plus.wrapping_sub(1),
                            base.wrapping_add(1),
                            plus,
                        ],
                    )
                } else {
                    let key = if raised {
                        b"drop_popup_chara_levelup1".as_slice()
                    } else {
                        b"drop_popup_chara_levelmax1".as_slice()
                    };
                    let pattern = query_localizable(ctx, &std_string_from_cstr(key));
                    let rarity = query_localizable(ctx, &rarity);

                    format_string3_int3(
                        ctx,
                        &pattern,
                        &reward_name,
                        &rarity,
                        &name,
                        [base.wrapping_add(1), base.wrapping_add(1), plus],
                    )
                }
            }
            _ => {
                let pattern =
                    query_localizable(ctx, &std_string_from_cstr(b"drop_popup_chara_limit"));
                let rarity = query_localizable(ctx, &rarity);

                format_string3(ctx, &pattern, &reward_name, &rarity, &name)
            }
        };
    }

    let next = cell.wrapping_add(1) as i64 as usize;
    let mut possession: Vec<u8> = Vec::new();

    if index & !0x10 != 6 {
        let pattern = query_localizable(ctx, b"item_possession");
        let held = *ctx.item_possession.entry(index).or_insert(0);
        let gained = xor_row46_get(ctx.bytes_from(row)?, next)
            .ok_or(Fault::index_out_of_range(next as i64, 0x2f))? as i32;

        possession = substitute_tokens(
            ctx,
            &pattern,
            &[
                (b"itemNum1".as_slice(), held.to_string().as_bytes()),
                (
                    b"itemNum2".as_slice(),
                    gained.wrapping_add(held).to_string().as_bytes(),
                ),
            ],
        )?;
    }

    let gained = xor_row46_get(ctx.bytes_from(row)?, next)
        .ok_or(Fault::index_out_of_range(next as i64, 0x2f))? as i32;
    let count = ctx.item_possession.entry(index).or_insert(0);

    *count = count.wrapping_add(gained);

    let shown = xor_row46_get(ctx.bytes_from(row)?, cell as i64 as usize)
        .ok_or(Fault::index_out_of_range(cell as i64, 0x2f))? as i32;
    let key = if shown == 6 || shown == 0xf7 || shown == 0xcf {
        b"drop_popup_xp".as_slice()
    } else {
        b"drop_popup_item".as_slice()
    };
    let pattern = query_localizable(ctx, key);
    let name = get_item_name(ctx, index);
    let amount = xor_row46_get(ctx.bytes_from(row)?, next)
        .ok_or(Fault::index_out_of_range(next as i64, 0x2f))? as i32;

    substitute_tokens(
        ctx,
        &pattern,
        &[
            (b"rewardName".as_slice(), reward_name.as_slice()),
            (b"itemName".as_slice(), name.as_slice()),
            (b"itemNum".as_slice(), amount.to_string().as_bytes()),
            (b"itemPossession".as_slice(), possession.as_slice()),
        ],
    )
}
