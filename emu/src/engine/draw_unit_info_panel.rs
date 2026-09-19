use crate::{operation, Fault};

use super::{
    ability_icon_is_base, draw_context, draw_cut, get_equipped_orb, get_orb_def, get_orb_slot_count, get_scene_id, get_talent_max_level, get_talent_trait_set,
    has_fixed_lineup, image_sprite_draw, orb_applies_to_unit, orb_trait_color, set_alpha, set_color, ui_node_add_child, ui_node_get_child, ui_node_set_alpha,
    ui_node_set_color, ui_node_set_sprite, ui_node_set_zoom, AppContext,
};

const SITE: &str = "draw_unit_info_panel";
const TRAIT_KEYS: [i32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

pub fn draw_unit_info_panel(ctx: &mut AppContext, unit_id: i32, form: i32, panel_x: i32, panel_y: i32) -> Result<(), Fault> {
    let fixed = get_scene_id(ctx)? == 0x12c && has_fixed_lineup(ctx, -1, -1, -1)?;
    let mut all_traits = false;

    for key in TRAIT_KEYS {
        if !ctx.trait_icons.get(&key).copied().unwrap_or(false) {
            all_traits = true;
        }
    }

    let mut wide = 0i32;
    let mut narrow = 0i32;

    for icon in 0..0x55 {
        let Some(row) = ctx.picture_book_abilities.iter().position(|columns| columns[0] == icon) else {
            continue;
        };

        if !ctx.ability_icons.get(&(row as i32)).copied().unwrap_or(false) {
            continue;
        }

        match ctx.picture_book_abilities[row][1] {
            0 => wide += 1,
            1 => narrow += 1,
            _ => {}
        }
    }

    let mut orbs = 0i32;

    if !fixed {
        let mut slot = 0i32;

        while slot < get_orb_slot_count(&ctx.orb_store, unit_id)? {
            if get_equipped_orb(ctx, unit_id, slot)? != -1 {
                orbs += 1;
            }

            slot += 1;
        }
    }

    let short = wide == 0 && all_traits;
    let icon_y;
    let mark_y;
    let row_x;

    if short {
        let span = narrow.wrapping_mul(0x2a);
        let half = operation::div_2(span.wrapping_add(-2));
        let left = panel_x.wrapping_sub(half).wrapping_add(-0x27);
        let shifted = orbs <= 0 || form < 2 || fixed;
        let baseline = if shifted { panel_y.wrapping_add(0x1a) } else { panel_y.wrapping_add(1) };
        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, sheet, left, baseline, 0xfc);

        let base = if narrow < 6 { 0 } else { operation::div_2(span.wrapping_add(-0xd2)) };

        icon_y = baseline.wrapping_add(-2);
        mark_y = baseline.wrapping_add(-3);
        row_x = base.wrapping_sub(half).wrapping_add(left).wrapping_add(0xb6);
    } else {
        let talented = get_talent_trait_set(ctx, unit_id, form)?;
        let order = ctx.picture_book_trait_order.clone();
        let base = panel_x.wrapping_add(-0x18a);

        for (index, key) in order.iter().enumerate() {
            let lit = ctx.trait_icons.get(key).copied().unwrap_or(false);

            if !lit {
                set_color(draw_context(&mut ctx.draw)?, 0x80, 0x80, 0x80, 0xff);
            }

            let cut = ctx
                .picture_book_traits
                .get(*key as i64 as usize)
                .and_then(|columns| columns.get(1))
                .copied()
                .ok_or(Fault::IndexOutOfRange { site: SITE, index: *key as i64, limit: ctx.picture_book_traits.len() as i64 })?;
            let sheet = ctx.img015_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let x = (index as i32).wrapping_mul(0x2b).wrapping_add(base);

            draw_cut(draw_context(&mut ctx.draw)?, sheet, x, panel_y, cut);
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

            if lit && talented.get(key).copied().unwrap_or(false) {
                let sheet = ctx.img015_sheet.clone();
                let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

                draw_cut(draw_context(&mut ctx.draw)?, sheet, x, panel_y, 0x10f);
            }
        }

        set_alpha(draw_context(&mut ctx.draw)?, 0x59);

        let page = ctx.i32_at(AppContext::SCENE_0X64_PAGE)?;
        let plate = if page == 7 {
            Some(0xfa)
        } else if get_scene_id(ctx)? == 0x12c || page == 3 || page == 9 {
            Some(0xfb)
        } else {
            None
        };

        if let Some(cut) = plate {
            let sheet = ctx.img015_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

            draw_cut(draw_context(&mut ctx.draw)?, sheet, panel_x.wrapping_add(0x38), panel_y.wrapping_add(-2), cut);
        }

        set_alpha(draw_context(&mut ctx.draw)?, 0xff);

        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, sheet, panel_x.wrapping_add(0x71), panel_y.wrapping_add(1), 0xf9);

        let mut column = 0i32;

        for icon in 0..0x55 {
            let Some(row) = ctx.picture_book_abilities.iter().position(|columns| columns[0] == icon) else {
                continue;
            };

            if !ctx.ability_icons.get(&(row as i32)).copied().unwrap_or(false) || ctx.picture_book_abilities[row][1] != 0 {
                continue;
            }

            let cut = ctx.picture_book_abilities[row][4];
            let sheet = ctx.img015_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
            let x = column.wrapping_mul(0x2a).wrapping_add(panel_x.wrapping_add(0xbd));

            draw_cut(draw_context(&mut ctx.draw)?, sheet, x, panel_y.wrapping_add(-1), cut);

            column = column.wrapping_add(1);

            if form < 2 || ctx.picture_book_abilities[row][2] == 0 || fixed {
                continue;
            }

            let abil = ctx.picture_book_abilities[row][2];
            let level = *ctx.talent_levels.entry(unit_id).or_default().entry(abil).or_default();

            if level == 0 {
                continue;
            }

            let capped = level == get_talent_max_level(ctx, 0, unit_id, abil)?;
            let sheet = ctx.img015_sheet.clone();
            let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

            draw_cut(draw_context(&mut ctx.draw)?, sheet, x, panel_y.wrapping_add(-2), 0x110 | i32::from(capped));
        }

        let span = narrow.wrapping_mul(0x2a);
        let half = operation::div_2(span.wrapping_add(-2));
        let inset = operation::div_2(orbs.wrapping_mul(0x33).wrapping_add(0x75));
        let left = panel_x.wrapping_sub(half).wrapping_add(-0x27).wrapping_sub(inset);
        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, sheet, left, panel_y.wrapping_add(0x38), 0xfc);

        let base = if narrow < 6 { 0 } else { operation::div_2(span.wrapping_add(-0xd2)) };

        icon_y = panel_y.wrapping_add(0x36);
        mark_y = panel_y.wrapping_add(0x34);
        row_x = base.wrapping_sub(half).wrapping_add(left).wrapping_add(0xb6);
    }

    let mut column = 0i32;

    for icon in 0..0x55 {
        let Some(row) = ctx.picture_book_abilities.iter().position(|columns| columns[0] == icon) else {
            continue;
        };

        if !ctx.ability_icons.get(&(row as i32)).copied().unwrap_or(false) || ctx.picture_book_abilities[row][1] != 1 {
            continue;
        }

        if !ability_icon_is_base(ctx, row as i32, unit_id, form)? {
            continue;
        }

        let cut = ctx.picture_book_abilities[row][4];
        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let x = column.wrapping_mul(0x2a).wrapping_add(row_x);

        draw_cut(draw_context(&mut ctx.draw)?, sheet, x, icon_y, cut);

        column = column.wrapping_add(1);

        if form < 2 || ctx.picture_book_abilities[row][2] == 0 || fixed {
            continue;
        }

        let abil = ctx.picture_book_abilities[row][2];
        let level = *ctx.talent_levels.entry(unit_id).or_default().entry(abil).or_default();

        if level == 0 {
            continue;
        }

        let capped = level == get_talent_max_level(ctx, 0, unit_id, abil)?;
        let sheet = ctx.img015_sheet.clone();
        let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        draw_cut(draw_context(&mut ctx.draw)?, sheet, x, mark_y, 0x110 | i32::from(capped));
    }

    if short || orbs <= 0 || form < 2 || fixed {
        return Ok(());
    }

    let plate_x = narrow.wrapping_mul(0x2a).wrapping_add(row_x).wrapping_add(0x1a);
    let sheet = ctx.img015_sheet.clone();
    let sheet = sheet.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

    draw_cut(draw_context(&mut ctx.draw)?, sheet, plate_x, icon_y, 0x122);

    let mut slot = 0i32;
    let mut column = 0i32;

    while slot < get_orb_slot_count(&ctx.orb_store, unit_id)? {
        let orb = get_equipped_orb(ctx, unit_id, slot)?;

        slot += 1;

        if orb == -1 {
            continue;
        }

        let x = column.wrapping_mul(0x33).wrapping_add(plate_x.wrapping_add(0x71));
        let sheet = ctx.img015_sheet.clone().ok_or(Fault::NullPointer { site: SITE })?;
        let mut node = ui_node_set_sprite(&sheet, x, 0, 0x123)?;

        ui_node_set_zoom(&mut node, 1.0, 1.0);

        let def = get_orb_def(&ctx.orb_store, orb)?;
        let trait_index = def.trait_index;
        let abil = def.abil;
        let grade = def.grade;

        for (source, cut) in [
            (ctx.equipment_attribute_sheet.clone(), trait_index),
            (ctx.equipment_effect_sheet.clone(), abil),
            (ctx.equipment_shadow_sheet.clone(), abil),
            (ctx.equipment_grade_sheet.clone(), grade),
        ] {
            let source = source.ok_or(Fault::NullPointer { site: SITE })?;
            let child = ui_node_set_sprite(&source, 0, 0, cut)?;
            let placed = ui_node_add_child(&mut node, child);

            ui_node_set_zoom(placed, 0.5764706, 0.5764706);
        }

        let color = orb_trait_color(trait_index);
        let tinted = ui_node_get_child(&mut node, 2)?;

        ui_node_set_color(tinted, color[0], color[1], color[2]);
        ui_node_set_alpha(tinted, 0x4b);

        if orb_applies_to_unit(ctx, abil, trait_index, unit_id)? != 2 {
            for child in [0, 1, 3] {
                let child = ui_node_get_child(&mut node, child)?;

                ui_node_set_color(child, 0x7f, 0x7f, 0x7f);
            }

            let child = ui_node_get_child(&mut node, 2)?;

            ui_node_set_color(child, operation::div_2(color[0]), operation::div_2(color[1]), operation::div_2(color[2]));
        }

        node.y = icon_y as f32;

        image_sprite_draw(ctx, &mut node)?;

        column = column.wrapping_add(1);
    }

    Ok(())
}
