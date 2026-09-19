use crate::Fault;

use super::{
    AppContext, has_talent, stat_area_attack, stat_attack_has_ld, stat_attack_only,
    stat_barrier_breaker_chance, stat_barrier_hitpoints, stat_base_destroyer, stat_behemoth_slayer,
    stat_colossus_slayer, stat_conjure_unit_id, stat_counter_surge, stat_critical_chance,
    stat_curse_chance, stat_curse_immune, stat_death_surge_chance, stat_dodge_chance,
    stat_double_bounty, stat_eva_killer, stat_explosion_chance, stat_explosion_immune,
    stat_freeze_chance, stat_freeze_immune, stat_has_omni_strike, stat_insane_damage,
    stat_insanely_tough, stat_is_metal, stat_knockback_chance, stat_knockback_immune,
    stat_massive_damage, stat_metal_killer_percent, stat_mini_surge_flag, stat_mini_wave_flag,
    stat_resist, stat_sage_slayer, stat_savage_blow_chance, stat_shield_pierce_chance,
    stat_shield_regen, stat_slow_chance, stat_slow_immune, stat_soulstrike,
    stat_strengthen_threshold, stat_strong_against, stat_surge_chance, stat_surge_immune,
    stat_survive, stat_toxic_chance, stat_toxic_immune, stat_warp_chance, stat_warp_immune,
    stat_wave_block, stat_wave_chance, stat_wave_immune, stat_weaken_chance, stat_weaken_immune,
    stat_witch_killer, stat_zombie_killer, trait_aku, trait_alien, trait_angel, trait_dark,
    trait_dojo, trait_floating, trait_metal, trait_red, trait_relic, trait_traitless, trait_zombie,
};

pub fn stat_has_ability(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
    ability: i32,
) -> Result<bool, Fault> {
    if ability as u32 > 0x52 {
        return Ok(false);
    }

    match ability {
        0x00 => Ok(stat_weaken_chance(ctx, faction, unit_id, form)? != 0),
        0x01 => Ok(stat_strengthen_threshold(ctx, faction, unit_id, form)? != 0),
        0x02 => Ok(stat_freeze_chance(ctx, faction, unit_id, form)? != 0),
        0x03 => Ok(stat_slow_chance(ctx, faction, unit_id, form)? != 0),
        0x04 => Ok(stat_survive(ctx, faction, unit_id, form)? != 0),
        0x05 => stat_base_destroyer(ctx, faction, unit_id, form),
        0x06 => Ok(stat_critical_chance(ctx, faction, unit_id, form)? != 0),
        0x07 => stat_attack_only(ctx, faction, unit_id, form),
        0x08 => stat_strong_against(ctx, faction, unit_id, form),
        0x09 => stat_resist(ctx, faction, unit_id, form),
        0x0a => stat_double_bounty(ctx, faction, unit_id, form),
        0x0b => stat_massive_damage(ctx, faction, unit_id, form),
        0x0c => Ok(stat_knockback_chance(ctx, faction, unit_id, form)? != 0),
        0x0d => {
            if stat_wave_chance(ctx, faction, unit_id, form, -1)? == 0 {
                return Ok(false);
            }

            Ok(stat_mini_wave_flag(ctx, faction, unit_id, form)? == 0)
        }
        0x0e => stat_is_metal(ctx, faction, unit_id, form),
        0x0f => stat_wave_immune(ctx, faction, unit_id, form),
        0x10 => stat_area_attack(ctx, faction, unit_id, form),
        0x11 => {
            if !stat_attack_has_ld(ctx, faction, unit_id, form, 0)? {
                return Ok(false);
            }

            Ok(!stat_has_omni_strike(ctx, faction, unit_id, form)?)
        }
        0x12 => stat_weaken_immune(ctx, faction, unit_id, form),
        0x13 => stat_freeze_immune(ctx, faction, unit_id, form),
        0x14 => stat_slow_immune(ctx, faction, unit_id, form),
        0x15 => stat_knockback_immune(ctx, faction, unit_id, form),
        0x16 => Ok(!stat_area_attack(ctx, faction, unit_id, form)?),
        0x17 => stat_witch_killer(ctx, faction, unit_id, form),
        0x18 => stat_zombie_killer(ctx, faction, unit_id, form),
        0x19 => stat_wave_block(ctx, faction, unit_id, form),
        0x1a => Ok(stat_barrier_breaker_chance(ctx, faction, unit_id, form)? != 0),
        0x1b => stat_warp_immune(ctx, faction, unit_id, form),
        0x1c => stat_has_omni_strike(ctx, faction, unit_id, form),
        0x1d => stat_eva_killer(ctx, faction, unit_id, form),
        0x1e => stat_curse_immune(ctx, faction, unit_id, form),
        0x1f => Ok(stat_warp_chance(ctx, faction, unit_id, form)? != 0),
        0x20 => has_talent(ctx, faction, unit_id, form, 0x12),
        0x21 => has_talent(ctx, faction, unit_id, form, 0x13),
        0x22 => has_talent(ctx, faction, unit_id, form, 0x14),
        0x23 => has_talent(ctx, faction, unit_id, form, 0x15),
        0x24 => has_talent(ctx, faction, unit_id, form, 0x16),
        0x25 => has_talent(ctx, faction, unit_id, form, 0x18),
        0x26 => has_talent(ctx, faction, unit_id, form, 0x19),
        0x27 => has_talent(ctx, faction, unit_id, form, 0x1a),
        0x28 => has_talent(ctx, faction, unit_id, form, 0x1b),
        0x29 => has_talent(ctx, faction, unit_id, form, 0x1c),
        0x2a => has_talent(ctx, faction, unit_id, form, 0x1e),
        0x2b => has_talent(ctx, faction, unit_id, form, 0x1f),
        0x2c => has_talent(ctx, faction, unit_id, form, 0x20),
        0x2d => stat_insane_damage(ctx, faction, unit_id, form),
        0x2e => stat_insanely_tough(ctx, faction, unit_id, form),
        0x2f => {
            if faction != 0 {
                return Ok(false);
            }

            trait_red(ctx, 0, unit_id, form, 1)
        }
        0x30 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_floating(ctx, 0, unit_id, form, 1)
        }
        0x31 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_dark(ctx, 0, unit_id, form, 1)
        }
        0x32 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_metal(ctx, 0, unit_id, form, 1)
        }
        0x33 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_angel(ctx, 0, unit_id, form, 1)
        }
        0x34 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_alien(ctx, 0, unit_id, form, 1)
        }
        0x35 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_zombie(ctx, 0, unit_id, form, 1)
        }
        0x36 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_relic(ctx, 0, unit_id, form, 1)
        }
        0x37 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_traitless(ctx, 0, unit_id, form, 1)
        }
        0x38 => Ok(stat_savage_blow_chance(ctx, faction, unit_id, form)? != 0),
        0x39 => Ok(stat_dodge_chance(ctx, faction, unit_id, form)? != 0),
        0x3a => {
            if stat_surge_chance(ctx, faction, unit_id, form)? == 0 {
                return Ok(false);
            }

            Ok(stat_mini_surge_flag(ctx, faction, unit_id, form)? == 0)
        }
        0x3b => stat_toxic_immune(ctx, faction, unit_id, form),
        0x3c => Ok(stat_curse_chance(ctx, faction, unit_id, form)? != 0),
        0x3d => stat_surge_immune(ctx, faction, unit_id, form),
        0x3e => {
            if stat_wave_chance(ctx, faction, unit_id, form, -1)? == 0 {
                return Ok(false);
            }

            Ok(stat_mini_wave_flag(ctx, faction, unit_id, form)? == 1)
        }
        0x3f => has_talent(ctx, faction, unit_id, form, 0x34),
        0x40 => has_talent(ctx, faction, unit_id, form, 0x36),
        0x41 => Ok(stat_shield_pierce_chance(ctx, faction, unit_id, form)? != 0),
        0x42 => {
            if faction != 0 {
                return Ok(false);
            }

            trait_aku(ctx, 0, unit_id, form, 1)
        }
        0x43 => stat_colossus_slayer(ctx, faction, unit_id, form),
        0x44 => stat_soulstrike(ctx, faction, unit_id, form),
        0x45 => stat_behemoth_slayer(ctx, faction, unit_id, form),
        0x46 => has_talent(ctx, faction, unit_id, form, 0x3d),
        0x47 => {
            if stat_surge_chance(ctx, faction, unit_id, form)? == 0 {
                return Ok(false);
            }

            Ok(stat_mini_surge_flag(ctx, faction, unit_id, form)? == 1)
        }
        0x48 => stat_counter_surge(ctx, faction, unit_id, form),
        0x49 => Ok(stat_conjure_unit_id(ctx, faction, unit_id, form)? >= 0),
        0x4a => stat_sage_slayer(ctx, faction, unit_id, form),
        0x4b => Ok(stat_metal_killer_percent(ctx, faction, unit_id, form)? != 0),
        0x4c => trait_dojo(ctx, faction, unit_id),
        0x4d => Ok(stat_toxic_chance(ctx, faction, unit_id)? != 0),
        0x4e => Ok(stat_death_surge_chance(ctx, faction, unit_id)? != 0),
        0x4f => Ok(stat_barrier_hitpoints(ctx, faction, unit_id)? != 0),
        0x50 => Ok(stat_shield_regen(ctx, faction, unit_id)? != 0),
        0x51 => Ok(stat_explosion_chance(ctx, faction, unit_id, form)? != 0),
        0x52 => stat_explosion_immune(ctx, faction, unit_id, form),
        _ => Ok(false),
    }
}
