use std::rc::Rc;

use crate::Fault;

use super::{
    format_localized, get_button_unit_row, get_sheet_table, get_unit_alt_art, get_unit_file_base, maanim_initialize, maanim_load, mamodel_get_part, mamodel_get_part_count, mamodel_load, mamodel_set_sheet_table,
    query_localizable, read_flag, stat_spawn_animation_flag, stat_use_gudetama_soul, std_map_int_maanim_subscript, string_format_text_int, texture_cache_load, trait_zombie, AppContext,
};

const SITE: &str = "load_unit_rig";

pub fn load_unit_rig(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    let buttons = if faction == 0 { 0x15 } else { 0xa };
    let side = if faction == 0 { 0usize } else { 1usize };

    for button in 0..buttons {
        if get_button_unit_row(ctx, faction, button)? == -1 {
            continue;
        }

        let row = get_button_unit_row(ctx, faction, button)?;
        let mut form = 0;

        if faction == 1 {
            if read_flag(ctx, AppContext::faction_flags(1))? & 1 != 0 {
                let unit_row = get_button_unit_row(ctx, 1, button)?;

                form = ctx.i32_at(AppContext::FACTION_1_UNIT_FORMS.wrapping_add((unit_row as i64 as usize).wrapping_mul(4)))?;
            }
        } else if faction == 0 && button as u32 <= 9 {
            form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + (button as usize) * 4)?;
        }

        let unit = row.wrapping_add(-2);
        let variant = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 1 { form } else { -1 };
        let base = get_unit_file_base(ctx, unit, variant)?;
        let name = format_localized(ctx, b"%s.png", &base)?;
        let png = query_localizable(ctx, &name);
        let name = format_localized(ctx, b"%s.imgcut", &base)?;
        let cut = query_localizable(ctx, &name);
        let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;
        let table = get_sheet_table(ctx, faction, button)?.ok_or(Fault::NullPointer { site: SITE })?;
        let slot = get_button_unit_row(ctx, faction, button)?.wrapping_add(-2);

        table.get(slot as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: slot as i64, limit: table.len() as i64 })?.set(sheet);

        if faction != 0 && faction != 1 {
            return Err(Fault::NullPointer { site: SITE });
        }

        let name = format_localized(ctx, b"%s.mamodel", &base)?;
        let path = query_localizable(ctx, &name);
        let limit = ctx.unit_models[side].len() as i64;
        let mut model = std::mem::take(ctx.unit_models[side].get_mut(button as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: button as i64, limit })?);

        mamodel_load(ctx, &mut model, &path)?;
        ctx.unit_models[side][button as usize] = model;

        for index in 0..4 {
            if button >= 0xb && index != 2 {
                continue;
            }

            let name = string_format_text_int(ctx, b"%s%02d.maanim", &base, index)?;
            let path = query_localizable(ctx, &name);
            let limit = ctx.unit_anims[side].len() as i64;
            let anims = ctx.unit_anims[side].get_mut(button as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: button as i64, limit })?;
            let mut anim = std::mem::take(std_map_int_maanim_subscript(anims, &index));

            maanim_load(ctx, &mut anim, &path)?;
            *std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &index) = anim;
        }

        if stat_spawn_animation_flag(ctx, faction, unit, form)? {
            let path = format_localized(ctx, b"%s_entry.maanim", &base)?;
            let mut anim = std::mem::take(std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &7));

            maanim_load(ctx, &mut anim, &path)?;
            *std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &7) = anim;
        }

        if stat_use_gudetama_soul(ctx, faction, unit, form)? {
            let path = format_localized(ctx, b"%s_soul.maanim", &base)?;
            let mut anim = std::mem::take(std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &8));

            maanim_load(ctx, &mut anim, &path)?;
            *std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &8) = anim;
        }

        if read_flag(ctx, AppContext::faction_flags(faction))? & 2 != 0 && trait_zombie(ctx, faction, unit, 0, 1)? {
            for stage in 0..3 {
                let key = stage + 4;
                let path = string_format_text_int(ctx, b"%s_zombie%02d.maanim", &base, stage)?;
                let mut anim = std::mem::take(std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &key));

                maanim_load(ctx, &mut anim, &path)?;
                *std_map_int_maanim_subscript(&mut ctx.unit_anims[side][button as usize], &key) = anim;
            }
        }

        if faction == 0 && get_unit_alt_art(ctx, unit, form)? != -1 {
            let model = &mut ctx.unit_models[0][button as usize];
            let mut part = 0;

            while part < mamodel_get_part_count(model) {
                let first = mamodel_get_part(model, 0).ok_or(Fault::NullPointer { site: SITE })?;
                let limit = model.parts.len() as i64;
                let target = model.parts.get_mut(first + part as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: part as i64, limit })?;

                if target.i32_at(0x24) != -1 {
                    target.set_i32_at(0x24, unit);
                }

                part += 1;
            }
        }

        let table = get_sheet_table(ctx, faction, button)?.unwrap_or_else(|| Rc::from([]));

        mamodel_set_sheet_table(&mut ctx.unit_models[side][button as usize], &table);
        maanim_initialize(&mut ctx.unit_models[side][button as usize], 0)?;
    }

    Ok(())
}
