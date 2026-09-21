use crate::{
    Fault,
    engine::{AppContext, obfuscate_value},
};

use super::{DECK_SLOTS, Setup};

const EMPTY_SLOT: i32 = -1;
const DECK_BIAS: i32 = 2;
const PLUS_SHIFT: u32 = 0x10;
const UNIT_OWNED: i32 = 1;
const TRUE_FORM: i32 = 2;
const ULTRA_FORM: i32 = 3;
const FORM_UNLOCKED: i32 = 2;
const TALENT_SLOTS: usize = 8;
const TALENT_STRIDE: usize = 0xe;
const TALENT_ABILITY: usize = 1;
const TALENT_MAX_LEVEL: usize = 2;
const SINGLE_LEVEL: i32 = 1;
const ITEM_STOCK: u32 = 1;

pub fn fill_dummy_lineup(ctx: &mut AppContext, setup: &Setup) -> Result<(), Fault> {
    for slot in 0..DECK_SLOTS {
        let row = setup
            .lineup
            .get(slot)
            .map_or(EMPTY_SLOT, |member| member.unit.wrapping_add(DECK_BIAS));

        ctx.set_i32_at(AppContext::DECK_PRESETS + slot * 4, row)?;
    }

    for member in setup.lineup.iter().take(DECK_SLOTS) {
        let unit = member.unit as usize;
        let mut cell = [0u8; 8];
        let packed = member.plus << PLUS_SHIFT | member.level.saturating_sub(1);

        cell[..4].copy_from_slice(&packed.to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::UNIT_LEVELS + unit * 8, cell)?;
        ctx.set_i32_at(AppContext::UNITS_OWNED + unit * 4, UNIT_OWNED)?;
        ctx.set_i32_at(AppContext::UNIT_FORMS + unit * 4, member.form)?;

        if member.form >= TRUE_FORM {
            ctx.set_i32_at(AppContext::REWARD_UNITS_OWNED + unit * 4, FORM_UNLOCKED)?;
        }

        if member.form >= ULTRA_FORM {
            ctx.set_i32_at(AppContext::REWARD_FORMS_OWNED + unit * 4, FORM_UNLOCKED)?;
        }
    }

    Ok(())
}

pub fn fill_dummy_talents(ctx: &mut AppContext, setup: &Setup) {
    for member in setup.lineup.iter().take(DECK_SLOTS) {
        if member.form < TRUE_FORM {
            continue;
        }

        for (slot, orb) in &member.orbs {
            ctx.equipped_orbs.entry(member.unit).or_default().insert(*slot, *orb);
        }

        let Some(definition) = ctx.talent_definitions.get(&member.unit).copied() else {
            continue;
        };
        let levels = ctx.talent_levels.entry(member.unit).or_default();

        for (ability, level) in &member.talents {
            let Some(slot) = (0..TALENT_SLOTS).find(|slot| definition[slot * TALENT_STRIDE + TALENT_ABILITY] == *ability) else {
                continue;
            };
            let highest = definition[slot * TALENT_STRIDE + TALENT_MAX_LEVEL].max(SINGLE_LEVEL);

            if *ability == 0 || *level <= 0 {
                continue;
            }

            levels.insert(*ability, (*level).min(highest));
        }
    }
}

pub fn stock_battle_items(ctx: &mut AppContext, setup: &Setup) -> Result<(), Fault> {
    for (item, stocked) in setup.items.iter().enumerate() {
        let mut cell = [0u8; 8];
        let held = if *stocked { ITEM_STOCK } else { 0 };

        cell[..4].copy_from_slice(&held.to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::ITEM_COUNTS_KIND_3 + item * 8, cell)?;
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED + item, [u8::from(*stocked)])?;
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED_SCORE_MODE + item, [u8::from(*stocked)])?;
    }

    Ok(())
}
