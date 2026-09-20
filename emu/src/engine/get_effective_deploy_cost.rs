use crate::{Fault, ops};

use super::{
    AppContext, get_button_unit_row, get_deploy_cost, get_global_map_id, get_special_rule_params,
    read_flag,
};

pub fn get_effective_deploy_cost(
    ctx: &mut AppContext,
    faction: i32,
    button: i32,
) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    let mut form = 0i32;
    let map_id = get_global_map_id(ctx, 0)?;
    let escalating = AppContext::faction_flags(faction)
        .wrapping_add(AppContext::WALLET_ESCALATING_COSTS)
        .wrapping_add((button as i64 as usize).wrapping_mul(4));

    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xb)?
        && ctx.i32_at(escalating)? != 0
    {
        let params = params.to_vec();
        let mode = *params.first().ok_or(Fault::index_out_of_range(0, 0))?;
        let mut total = ctx.i32_at(escalating)?;
        let scaled;

        if mode == 0 {
            let unit_id = get_button_unit_row(ctx, faction, button)?.wrapping_sub(2);
            let mut rule_form = 0i32;

            if faction == 1 {
                if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
                    let row = get_button_unit_row(ctx, 1, button)?;

                    rule_form = ctx.i32_at(
                        ((row as i64) * 4 + AppContext::FACTION_1_UNIT_FORMS as i64) as usize,
                    )?;
                }
            } else if faction == 0 && button as u32 <= 9 {
                rule_form =
                    ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as u32 as usize) * 4)?;
            }

            let cost = get_deploy_cost(ctx, unit_id, rule_form, 1, button)?;

            if params.len() <= 1 {
                return Err(Fault::index_out_of_range(1, params.len() as i64));
            }

            scaled = Some(cost as i64);
        } else if mode == 1 {
            if params.len() <= 1 {
                return Err(Fault::index_out_of_range(1, params.len() as i64));
            }

            scaled = Some(ctx.i32_at(escalating)? as i64);
        } else {
            scaled = None;
        }

        if let Some(amount) = scaled {
            let step = (ops::div_100(amount) as i32).wrapping_mul(params[1]);
            let rounded = (ops::div_100(step as i64) as i32).wrapping_mul(0x64);

            total = total.wrapping_add(step);
            total = total.wrapping_add(rounded.wrapping_sub(step));
        }

        if params.len() <= 2 {
            return Err(Fault::index_out_of_range(2, params.len() as i64));
        }

        let cap = params[2].wrapping_mul(0x64);

        return Ok(if total < cap { total } else { cap });
    }

    let unit_id = get_button_unit_row(ctx, faction, button)?.wrapping_sub(2);

    if faction == 1 {
        if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
            let row = get_button_unit_row(ctx, 1, button)?;

            form =
                ctx.i32_at(((row as i64) * 4 + AppContext::FACTION_1_UNIT_FORMS as i64) as usize)?;
        }
    } else if faction == 0 && button as u32 <= 9 {
        form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as u32 as usize) * 4)?;
    }

    get_deploy_cost(ctx, unit_id, form, 1, button)
}
