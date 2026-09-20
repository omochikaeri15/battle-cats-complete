use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, TALENT_ABILITY_ICONS, get_talent_icon_state, has_orb, stat_area_attack,
    stat_attack_has_ld, stat_attack_only, stat_barrier_breaker_chance, stat_barrier_hitpoints,
    stat_base_destroyer, stat_behemoth_slayer, stat_colossus_slayer, stat_conjure_unit_id,
    stat_counter_surge, stat_critical_chance, stat_curse_chance, stat_curse_immune,
    stat_death_surge_chance, stat_dodge_chance, stat_double_bounty, stat_drain_chance,
    stat_drain_immune, stat_eva_killer, stat_explosion_chance, stat_explosion_immune,
    stat_freeze_chance, stat_freeze_immune, stat_has_omni_strike, stat_has_shield,
    stat_insane_damage, stat_insanely_tough, stat_is_metal, stat_knockback_chance,
    stat_knockback_immune, stat_massive_damage, stat_metal_killer_percent, stat_mini_surge_flag,
    stat_mini_wave_flag, stat_resist, stat_sage_slayer, stat_savage_blow_chance,
    stat_shield_pierce_chance, stat_slow_chance, stat_slow_immune, stat_soulstrike,
    stat_strengthen_boost, stat_strong_against, stat_surge_chance, stat_surge_immune, stat_survive,
    stat_toxic_chance, stat_toxic_immune, stat_warp_chance, stat_warp_immune, stat_wave_block,
    stat_wave_chance, stat_wave_immune, stat_weaken_chance, stat_weaken_immune, stat_witch_killer,
    stat_zombie_killer, trait_dojo,
};

pub fn get_ability_icon_set(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    with_talents: u8,
) -> Result<BTreeMap<i32, bool>, Fault> {
    let mut icons: BTreeMap<i32, bool> = BTreeMap::new();

    if faction == 1 {
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

    icons.insert(0x0, stat_weaken_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(
        0x1,
        stat_strengthen_boost(ctx, faction, unit_id, form)? != 0,
    );
    icons.insert(0x2, stat_freeze_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x3, stat_slow_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x4, stat_survive(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x5, stat_base_destroyer(ctx, faction, unit_id, form)?);
    icons.insert(0x6, stat_critical_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x7, stat_attack_only(ctx, faction, unit_id, form)?);
    icons.insert(0x8, stat_strong_against(ctx, faction, unit_id, form)?);
    icons.insert(0x9, stat_resist(ctx, faction, unit_id, form)?);
    icons.insert(0xa, stat_double_bounty(ctx, faction, unit_id, form)?);
    icons.insert(0xb, stat_massive_damage(ctx, faction, unit_id, form)?);
    icons.insert(
        0xc,
        stat_knockback_chance(ctx, faction, unit_id, form)? != 0,
    );
    let level = stat_wave_chance(ctx, faction, unit_id, form, -1)?;
    let shown = if level != 0 {
        let mini = stat_mini_wave_flag(ctx, faction, unit_id, form)?;

        if faction != 0 {
            mini == 0
        } else if mini != 0 {
            get_talent_icon_state(ctx, 0, unit_id, form, 0x11)? != 0
        } else {
            true
        }
    } else if faction != 0 {
        false
    } else {
        get_talent_icon_state(ctx, 0, unit_id, form, 0x11)? != 0
    };

    icons.insert(0xd, shown);

    icons.insert(0xf, stat_wave_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x10, stat_area_attack(ctx, faction, unit_id, form)?);
    let shown = stat_attack_has_ld(ctx, faction, unit_id, form, 0)?
        && !stat_has_omni_strike(ctx, faction, unit_id, form)?;

    icons.insert(0x11, shown);

    icons.insert(0x12, stat_weaken_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x13, stat_freeze_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x14, stat_slow_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x15, stat_knockback_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x16, !stat_area_attack(ctx, faction, unit_id, form)?);
    icons.insert(0x17, stat_witch_killer(ctx, faction, unit_id, form)?);
    icons.insert(0x18, stat_zombie_killer(ctx, faction, unit_id, form)?);
    icons.insert(0x19, stat_wave_block(ctx, faction, unit_id, form)?);
    icons.insert(
        0x1a,
        stat_barrier_breaker_chance(ctx, faction, unit_id, form)? != 0,
    );
    icons.insert(0x1b, stat_warp_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x1c, stat_has_omni_strike(ctx, faction, unit_id, form)?);
    icons.insert(0x1d, stat_eva_killer(ctx, faction, unit_id, form)?);
    icons.insert(0x1e, stat_curse_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x1f, stat_warp_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x2d, stat_insane_damage(ctx, faction, unit_id, form)?);
    icons.insert(0x2e, stat_insanely_tough(ctx, faction, unit_id, form)?);
    icons.insert(
        0x38,
        stat_savage_blow_chance(ctx, faction, unit_id, form)? != 0,
    );
    icons.insert(0x39, stat_dodge_chance(ctx, faction, unit_id, form)? != 0);
    let level = stat_surge_chance(ctx, faction, unit_id, form)?;
    let shown = if level != 0 {
        let mini = stat_mini_surge_flag(ctx, faction, unit_id, form)?;

        if faction != 0 {
            mini == 0
        } else if mini != 0 {
            get_talent_icon_state(ctx, 0, unit_id, form, 0x38)? != 0
        } else {
            true
        }
    } else if faction != 0 {
        false
    } else {
        get_talent_icon_state(ctx, 0, unit_id, form, 0x38)? != 0
    };

    icons.insert(0x3a, shown);

    icons.insert(0x3b, stat_toxic_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x3c, stat_curse_chance(ctx, faction, unit_id, form)? != 0);
    icons.insert(0x3d, stat_surge_immune(ctx, faction, unit_id, form)?);
    let level = stat_wave_chance(ctx, faction, unit_id, form, -1)?;
    let shown = level != 0 && stat_mini_wave_flag(ctx, faction, unit_id, form)? == 1;

    icons.insert(0x3e, shown);

    icons.insert(
        0x41,
        stat_shield_pierce_chance(ctx, faction, unit_id, form)? != 0,
    );
    let slayer = stat_colossus_slayer(ctx, faction, unit_id, form)?;
    let orb = form >= 2 && has_orb(ctx, &ctx.orb_store, unit_id, 0xa)?;

    icons.insert(0x43, slayer || orb);

    icons.insert(0x44, stat_soulstrike(ctx, faction, unit_id, form)?);
    icons.insert(0x45, stat_behemoth_slayer(ctx, faction, unit_id, form)?);
    let level = stat_surge_chance(ctx, faction, unit_id, form)?;
    let shown = if level != 0 {
        let mini = stat_mini_surge_flag(ctx, faction, unit_id, form)?;

        if faction != 0 {
            mini == 1
        } else if mini != 1 {
            get_talent_icon_state(ctx, 0, unit_id, form, 0x41)? != 0
        } else {
            true
        }
    } else if faction != 0 {
        false
    } else {
        get_talent_icon_state(ctx, 0, unit_id, form, 0x41)? != 0
    };

    icons.insert(0x47, shown);

    icons.insert(0x48, stat_counter_surge(ctx, faction, unit_id, form)?);
    icons.insert(
        0x49,
        stat_conjure_unit_id(ctx, faction, unit_id, form)? >= 0,
    );
    icons.insert(0x4a, stat_sage_slayer(ctx, faction, unit_id, form)?);
    icons.insert(
        0x4b,
        stat_metal_killer_percent(ctx, faction, unit_id, form)? != 0,
    );
    icons.insert(0x4c, trait_dojo(ctx, faction, unit_id)?);
    icons.insert(0x4d, stat_toxic_chance(ctx, faction, unit_id)? != 0);
    icons.insert(0x4e, stat_death_surge_chance(ctx, faction, unit_id)? != 0);
    icons.insert(0x4f, stat_barrier_hitpoints(ctx, faction, unit_id)? != 0);
    icons.insert(0x50, stat_has_shield(ctx, faction, unit_id)?);
    icons.insert(
        0x51,
        stat_explosion_chance(ctx, faction, unit_id, form)? != 0,
    );
    icons.insert(0x52, stat_explosion_immune(ctx, faction, unit_id, form)?);
    icons.insert(0x53, stat_drain_chance(ctx, faction, unit_id)? != 0);
    icons.insert(0x54, stat_drain_immune(ctx, faction, unit_id, form)?);
    if faction == 0 {
        icons.insert(0xe, stat_is_metal(ctx, 0, unit_id, form)?);
    }

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x12)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0x15)?
    } else {
        talent != 0
    };

    icons.insert(0x20, shown);

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x13)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0x14)?
    } else {
        talent != 0
    };

    icons.insert(0x21, shown);

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x14)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0xe)?
    } else {
        talent != 0
    };

    icons.insert(0x22, shown);

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x15)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0x8)?
    } else {
        talent != 0
    };

    icons.insert(0x23, shown);

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x16)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0x6)?
    } else {
        talent != 0
    };

    icons.insert(0x24, shown);

    icons.insert(
        0x25,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x18)? != 0,
    );
    icons.insert(
        0x26,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x19)? != 0,
    );
    icons.insert(
        0x27,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x1a)? != 0,
    );
    icons.insert(
        0x28,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x1b)? != 0,
    );
    icons.insert(
        0x29,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x1c)? != 0,
    );
    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x1e)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0xf)?
    } else {
        talent != 0
    };

    icons.insert(0x2a, shown);

    icons.insert(
        0x2b,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x1f)? != 0,
    );
    icons.insert(
        0x2c,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x20)? != 0,
    );
    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x34)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0xc)?
    } else {
        talent != 0
    };

    icons.insert(0x3f, shown);

    let talent = get_talent_icon_state(ctx, 0, unit_id, form, 0x36)?;
    let shown = if form >= 2 && talent == 0 {
        has_orb(ctx, &ctx.orb_store, unit_id, 0x17)?
    } else {
        talent != 0
    };

    icons.insert(0x40, shown);

    icons.insert(
        0x46,
        get_talent_icon_state(ctx, 0, unit_id, form, 0x3d)? != 0,
    );

    if with_talents != 0 {
        for slot in 0..8usize {
            let abil = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71])[1 + slot * 14];

            if let Some(&(_, icon)) = TALENT_ABILITY_ICONS.iter().find(|(key, _)| *key == abil) {
                icons.insert(icon, true);
            }
        }
    }

    if !icons.values().any(|shown| *shown) {
        icons.insert(0x55, true);
    }

    Ok(icons)
}
