use std::collections::BTreeMap;

use crate::{operation, Fault};

use super::{
    conjurer_on_field, cos_deg, draw_context, draw_cooldown_bar, draw_cut, draw_cut_scaled, draw_deck_preset_mark, draw_deploy_cost, draw_model, draw_panel,
    fill_rect, get_button_unit_form, get_button_unit_id, get_button_unit_row, get_current_stage_id, get_deck_cooldown, get_drawable_width,
    get_effective_deploy_cost, get_equipped_orb, get_global_map_id, get_money, get_orb_def, get_orb_slot_count, get_setting, get_special_rule, get_unit_rarity,
    glow_set, imgcut_get_sprite_cut, is_deploy_blocked, maanim_execute, orb_ability_flag, orb_ability_repeat, orb_icon_visible, set_alpha, set_color, set_tint,
    set_tint_alpha, slot_conjure_ready, slot_has_flagged_orb, stage_has_restriction, unit_meets_restriction, AppContext, DECK_BASE_Y,
    DECK_PRESS_SIZE_TABLE, DECK_SLOT_X_TABLE,
};

const SITE: &str = "draw_deck_button";

fn seat(index: i32) -> Result<i32, Fault> {
    DECK_SLOT_X_TABLE
        .get(index as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: index as i64, limit: DECK_SLOT_X_TABLE.len() as i64 })
}

fn press_size(ctx: &AppContext, index: i32) -> Result<i32, Fault> {
    let step = ctx.i32_at(AppContext::DECK_PRESS.wrapping_add(((index as i64) * 4) as usize))?;

    DECK_PRESS_SIZE_TABLE
        .get(step as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: DECK_PRESS_SIZE_TABLE.len() as i64 })
}

pub fn draw_deck_button(ctx: &mut AppContext, slot: i32, layer: i32, overlay: u8) -> Result<(), Fault> {
    let mut bottom = DECK_BASE_Y
        .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
        .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let column = slot.wrapping_sub(operation::div_5(slot) * 5);
    let mut width = 0x6e;
    let mut height = 0x55;
    let across;
    let down;
    let origin;

    if layer == 3 {
        let base = seat(column)? as f64;
        let mut span = get_drawable_width(ctx)?;

        if slot <= 4 {
            bottom = bottom.wrapping_sub(get_setting(&ctx.settings, b"battle_slot_2lines_line", 0x5a)?);
        }

        span = span.wrapping_add(-0x3c0);

        let base = operation::cvttsd2si(span as f64 * 0.5 + base);
        let size = press_size(ctx, slot)?;
        let half = operation::div_2(size);

        origin = base;
        across = base.wrapping_sub(half);
        down = bottom.wrapping_sub(half);
        width = size.wrapping_add(0x6e);
        height = size.wrapping_add(0x55);
    } else {
        let base = seat(slot)? as f64;
        let span = get_drawable_width(ctx)?.wrapping_add(-0x3c0);
        let base = span as f64 * 0.5 + base;

        if layer == 2 {
            let shown = ctx.i32_at(AppContext::DECK_ROW_SHOWN)?;

            if shown == 1 {
                if slot <= 4 {
                    bottom = bottom.wrapping_add(ctx.i32_at(AppContext::DECK_ROW_SWAP_OFFSETS + 4)?);
                } else {
                    bottom = bottom.wrapping_add(ctx.i32_at(AppContext::DECK_ROW_SWAP_OFFSETS)?);
                }
            } else if shown == 0 {
                if slot < 5 {
                    bottom = bottom.wrapping_add(ctx.i32_at(AppContext::DECK_ROW_SWAP_OFFSETS)?);
                } else {
                    bottom = bottom.wrapping_add(ctx.i32_at(AppContext::DECK_ROW_SWAP_OFFSETS + 4)?);
                }
            }

            origin = operation::cvttsd2si(base);
            across = origin;
            down = bottom;
        } else if layer == 1 {
            bottom = bottom.wrapping_add(0xc);
            origin = operation::cvttsd2si(base);
            across = origin;
            down = bottom;
        } else if layer != 0 {
            origin = operation::cvttsd2si(base);
            across = origin;
            down = bottom;
        } else {
            let base = operation::cvttsd2si(base);
            let size = press_size(ctx, column)?;
            let half = operation::div_2(size);

            origin = base;
            across = base.wrapping_sub(half);
            down = bottom.wrapping_sub(half);
            width = size.wrapping_add(0x6e);
            height = size.wrapping_add(0x55);
        }
    }

    let conjure_ready = slot_conjure_ready(ctx, 0, slot)?;
    let dim;
    let state;

    if get_button_unit_id(ctx, 0, slot)? < 0 {
        state = i32::from((layer.wrapping_add(-1) as u32) < 2) * 3;
        dim = true;
    } else if conjure_ready {
        if conjurer_on_field(ctx, 0, slot, 0)? {
            set_color(draw_context(&mut ctx.draw)?, 0x9d, 0xd7, 0xff, 0xff);

            state = 2;
            dim = false;
        } else {
            set_color(draw_context(&mut ctx.draw)?, 0x87, 0x64, 0x64, 0xff);

            state = 1;
            dim = false;
        }
    } else {
        let stage = get_current_stage_id(ctx)?;
        let restricted = stage_has_restriction(ctx, &ctx.stage_restrictions, stage)? && !unit_meets_restriction(ctx, slot, 1)?;

        if restricted || is_deploy_blocked(ctx, slot)? {
            let map = get_global_map_id(ctx, 0)?;

            dim = get_special_rule(ctx, &ctx.special_rules, map, 0)?;

            set_color(draw_context(&mut ctx.draw)?, 0x87, 0x64, 0x64, 0xff);

            state = 1;
        } else if get_deck_cooldown(ctx, AppContext::faction_flags(0), slot)? != 0 {
            dim = true;
            state = 3;
        } else {
            let wallet = AppContext::faction_flags(0);
            let money = get_money(ctx, wallet)?;
            let cost = get_effective_deploy_cost(ctx, 0, slot)?;

            dim = true;

            if (layer.wrapping_add(-1) as u32) < 2 {
                state = 3;
            } else if money >= cost {
                state = 0;
            } else {
                state = 3;
            }
        }
    }

    let icon = ctx
        .unit_icon_textures
        .get(slot as i64 as usize)
        .ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: ctx.unit_icon_textures.len() as i64 })?
        .clone();
    let sheet = icon.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, across, down, width, height, 0);
    set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    if state == 3 {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);
        set_tint_alpha(draw_context(&mut ctx.draw)?, 0x64);
        fill_rect(draw_context(&mut ctx.draw)?, across, down, width, height);
    }

    let unit = get_button_unit_id(ctx, 0, slot)?;

    if unit < 0 {
        return Ok(());
    }

    let wallet = AppContext::faction_flags(0);

    if conjure_ready && conjurer_on_field(ctx, 0, slot, 0)? {
        let timer = ctx.i32_at(wallet.wrapping_add(AppContext::WALLET_CONJURE_TIMER).wrapping_add(((slot as i64) * 4) as usize))?;

        if timer <= 0x22 && timer.wrapping_sub(operation::div_7(timer) * 7) <= 2 {
            glow_set(draw_context(&mut ctx.draw)?, 1);
            set_tint(draw_context(&mut ctx.draw)?, 0x28, 0x28, 0x28, 0xff);
            fill_rect(draw_context(&mut ctx.draw)?, across, down, width, height);
            glow_set(draw_context(&mut ctx.draw)?, 0);
        }
    }

    draw_deck_preset_mark(ctx, slot, across, down, width, height)?;

    if (layer.wrapping_add(-1) as u32) < 2 {
        return Ok(());
    }

    if slot <= 4 && overlay == 0 {
        let left = across.wrapping_add(-8);
        let top = down.wrapping_add(-8);

        for entry in 0..5i64 {
            let banner = ctx.i32_at(AppContext::COMBO_BANNER_UNITS.wrapping_add((entry * 4) as usize))?;

            if banner != get_button_unit_row(ctx, 0, slot)?.wrapping_add(-2) {
                continue;
            }

            let ticks = ctx.i32_at(AppContext::COMBO_BANNER_TICKS)?;

            if (ticks.wrapping_sub(operation::div_4(ticks) * 4).wrapping_add(1) as u32) > 2 {
                continue;
            }

            let sheet = ctx.img002_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let size = press_size(ctx, column)?;

            draw_cut_scaled(draw_context(&mut ctx.draw)?, sheet, left, top, size.wrapping_add(0x7e), size.wrapping_add(0x65), 0x2c);
        }
    }

    if state == 2 {
        let timer = ctx.i32_at(wallet.wrapping_add(AppContext::WALLET_CONJURE_TIMER).wrapping_add(((slot as i64) * 4) as usize))?;
        let glow = operation::cvttss2si(cos_deg(timer.wrapping_mul(10) as f32) * 31.0 + 224.0);

        set_color(draw_context(&mut ctx.draw)?, glow, glow, 0xff, 0xff);

        let sheet = ctx.img002_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let cut = imgcut_get_sprite_cut(sheet, 0x35)?;
        let x = origin.wrapping_add(operation::div_2(0x6ei32.wrapping_sub(cut[2])));
        let y = bottom.wrapping_add(operation::div_2(0x55i32.wrapping_sub(cut[3])));

        draw_cut(draw_context(&mut ctx.draw)?, sheet, x, y, 0x35);
        set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
    }

    if overlay != 0 && ctx.i32_at(AppContext::UNIT_INFO_SLOT)? == slot {
        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let half = operation::div_2(ctx.i32_at(AppContext::BATTLE_TICKS)?);
        let phase = half.wrapping_sub(operation::div_2(half) * 2);

        draw_panel(
            draw_context(&mut ctx.draw)?,
            sheet,
            origin.wrapping_add(-1),
            bottom.wrapping_add(-1),
            0x70,
            0x57,
            1.0,
            0x159i32.wrapping_add(phase * 2),
            0x15ai32.wrapping_add(phase * 2),
        );
    }

    if get_deck_cooldown(ctx, wallet, slot)? != 0 {
        draw_cooldown_bar(ctx, slot)?;
    } else if dim {
        let mode = i32::from(state == 3);
        let money_sheet = ctx.img001_sheet.clone();
        let alt = ctx.deploy_cost_alt_sheet.clone();
        let cost = get_effective_deploy_cost(ctx, 0, slot)?;
        let sheet = money_sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_deploy_cost(
            draw_context(&mut ctx.draw)?,
            sheet,
            alt.as_deref(),
            operation::div_100(cost),
            origin.wrapping_add(0x5a),
            bottom.wrapping_add(0x32),
            mode,
            0,
            0,
        )?;
    }

    let map = get_global_map_id(ctx, 0)?;

    if get_special_rule(ctx, &ctx.special_rules, map, 3)? {
        let money_sheet = ctx.img001_sheet.clone();
        let sheet = money_sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let cut = imgcut_get_sprite_cut(sheet, 0x87)?;
        let x = origin.wrapping_sub(cut[2]).wrapping_add(0x73);
        let y = bottom.wrapping_add(-5);
        let rarity = get_unit_rarity(ctx, unit)?;

        draw_cut(draw_context(&mut ctx.draw)?, sheet, x, y, 0x87i32.wrapping_add(rarity));
    }

    if get_button_unit_form(ctx, 0, slot)? < 2 {
        return Ok(());
    }

    if !slot_has_flagged_orb(ctx, 0, slot)? {
        return Ok(());
    }

    if get_deck_cooldown(ctx, wallet, slot)? != 0 {
        return Ok(());
    }

    if overlay == 0 && ctx.i32_at(wallet.wrapping_add(AppContext::WALLET_SLOT_FLASH).wrapping_add(((slot as i64) * 4) as usize))? >= 0 {
        maanim_execute(&mut ctx.invoke_equipment_model, Some(&ctx.invoke_equipment_anim), 0, 0)?;

        let x = across.wrapping_add(operation::div_2(width));
        let y = down.wrapping_add(operation::div_2(height));
        let model = std::mem::take(&mut ctx.invoke_equipment_model);

        draw_model(draw_context(&mut ctx.draw)?, &model, x, y);

        ctx.invoke_equipment_model = model;
    }

    let mut seen: BTreeMap<i32, u8> = BTreeMap::new();
    let upper = bottom.wrapping_add(-5);
    let lower = bottom.wrapping_add(-2);
    let mut index = 0i32;

    while index < get_orb_slot_count(&ctx.orb_store, unit)? {
        let orb = get_equipped_orb(ctx, unit, index)?;

        if orb == -1 {
            index = index.wrapping_add(1);

            continue;
        }

        let def = get_orb_def(&ctx.orb_store, orb)?;
        let trait_index = def.trait_index;
        let abil = def.abil;

        if !orb_icon_visible(ctx, wallet, 0, slot, index, 0)? {
            index = index.wrapping_add(1);

            continue;
        }

        if !orb_ability_flag(&mut ctx.orb_store, abil) {
            index = index.wrapping_add(1);

            continue;
        }

        if !orb_ability_repeat(&mut ctx.orb_store, abil) {
            if seen.get(&abil).copied().unwrap_or(0) != 0 {
                index = index.wrapping_add(1);

                continue;
            }

            seen.insert(abil, 1);
        }

        let attribute = ctx.equipment_attribute_s_sheet.clone();
        let attribute = attribute.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let step = index.wrapping_mul(27);

        draw_cut(draw_context(&mut ctx.draw)?, attribute, origin.wrapping_add(-5).wrapping_add(step), upper, trait_index);

        let effect = ctx.equipment_effect_s_sheet.clone();
        let effect = effect.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, effect, step.wrapping_add(origin).wrapping_add(-2), lower, abil);
        set_alpha(draw_context(&mut ctx.draw)?, 0xff);

        index = index.wrapping_add(1);
    }

    Ok(())
}
