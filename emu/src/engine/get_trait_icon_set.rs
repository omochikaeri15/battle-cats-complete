use std::collections::BTreeMap;

use crate::Fault;

use super::{
    trait_aku, trait_alien, trait_angel, trait_behemoth, trait_colossus, trait_dark, trait_eva, trait_floating, trait_kaijin, trait_metal, trait_red,
    trait_relic, trait_sage, trait_traitless, trait_witch, trait_zombie, AppContext, TALENT_TRAIT_ICONS,
};

pub fn get_trait_icon_set(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32, with_talents: u8) -> Result<BTreeMap<i32, bool>, Fault> {
    let mut icons: BTreeMap<i32, bool> = BTreeMap::new();
    let mut enemy_side = false;

    if faction == 1 {
        enemy_side = true;

        if (unit_id.wrapping_add(2) as u32) >= 0x324 {
            return Ok(icons);
        }
    } else if faction == 0 {
        if (unit_id.wrapping_add(2) as u32) > 0x36d {
            return Ok(icons);
        }

        if (form as u32) >= 4 {
            return Ok(icons);
        }
    }

    icons.insert(0, trait_red(ctx, faction, unit_id, form, 1)?);
    icons.insert(1, trait_floating(ctx, faction, unit_id, form, 1)?);
    icons.insert(2, trait_dark(ctx, faction, unit_id, form, 1)?);
    icons.insert(3, trait_metal(ctx, faction, unit_id, form, 1)?);
    icons.insert(4, trait_angel(ctx, faction, unit_id, form, 1)?);
    icons.insert(5, trait_alien(ctx, faction, unit_id, form, 1)?);
    icons.insert(6, trait_zombie(ctx, faction, unit_id, form, 1)?);
    icons.insert(7, trait_relic(ctx, faction, unit_id, form, 1)?);
    icons.insert(8, trait_aku(ctx, faction, unit_id, form, 1)?);
    icons.insert(9, trait_traitless(ctx, faction, unit_id, form, 1)?);

    if enemy_side {
        icons.insert(0xa, trait_colossus(ctx, 1, unit_id)?);
        icons.insert(0xb, trait_behemoth(ctx, 1, unit_id)?);
        icons.insert(0xc, trait_sage(ctx, 1, unit_id)?);
        icons.insert(0xd, trait_witch(ctx, 1, unit_id, form, 1)?);
        icons.insert(0xe, trait_eva(ctx, 1, unit_id, form, 1)?);
        icons.insert(0xf, trait_kaijin(ctx, 1, unit_id)?);
    }

    if with_talents != 0 {
        for slot in 0..8usize {
            let abil = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71])[1 + slot * 14];

            if let Some(&(_, icon)) = TALENT_TRAIT_ICONS.iter().find(|(key, _)| *key == abil) {
                icons.insert(icon, true);
            }
        }
    }

    if !icons.values().any(|shown| *shown) {
        icons.insert(0x10, true);
    }

    Ok(icons)
}
