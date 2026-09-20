use crate::{
    Fault,
    engine::{AppContext, obfuscate_value},
};

const DECK_SLOTS: usize = 10;
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
const TALENT_ULTRA: usize = 0xe;
const HITPOINTS_TALENT: i32 = 0x20;
const SINGLE_LEVEL: i32 = 1;
const BATTLE_ITEMS: usize = 6;
const ITEM_STOCK: u32 = 1;
const FIRST_ORB_SLOT: i32 = 0;
const STRENGTHEN_A: i32 = 273;

#[derive(Clone, Copy)]
enum Talents {
    None,
    One(i32, i32),
    Normal,
    Every,
}

#[derive(Clone, Copy)]
struct Member {
    unit: i32,
    form: i32,
    level: u32,
    plus: u32,
    talents: Talents,
    orb: Option<i32>,
}

const LINEUP: [Member; 9] = [
    Member { unit: 92, form: 2, level: 50, plus: 0, talents: Talents::None, orb: None },
    Member { unit: 148, form: 2, level: 50, plus: 0, talents: Talents::One(HITPOINTS_TALENT, 10), orb: None },
    Member { unit: 834, form: 1, level: 30, plus: 0, talents: Talents::None, orb: None },
    Member { unit: 397, form: 3, level: 60, plus: 0, talents: Talents::None, orb: None },
    Member { unit: 273, form: 1, level: 40, plus: 0, talents: Talents::None, orb: None },
    Member { unit: 32, form: 2, level: 50, plus: 30, talents: Talents::Normal, orb: None },
    Member { unit: 439, form: 2, level: 60, plus: 0, talents: Talents::Every, orb: Some(STRENGTHEN_A) },
    Member { unit: 830, form: 1, level: 40, plus: 0, talents: Talents::None, orb: None },
    Member { unit: 769, form: 1, level: 50, plus: 0, talents: Talents::None, orb: None },
];

pub fn fill_dummy_lineup(ctx: &mut AppContext) -> Result<(), Fault> {
    for slot in 0..DECK_SLOTS {
        let row = LINEUP
            .get(slot)
            .map_or(EMPTY_SLOT, |member| member.unit.wrapping_add(DECK_BIAS));

        ctx.set_i32_at(AppContext::DECK_PRESETS + slot * 4, row)?;
    }

    for member in LINEUP {
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

pub fn fill_dummy_talents(ctx: &mut AppContext) {
    for member in LINEUP {
        if let Some(orb) = member.orb {
            ctx.equipped_orbs
                .entry(member.unit)
                .or_default()
                .insert(FIRST_ORB_SLOT, orb);
        }

        let Some(definition) = ctx.talent_definitions.get(&member.unit).copied() else {
            continue;
        };
        let levels = ctx.talent_levels.entry(member.unit).or_default();

        for slot in 0..TALENT_SLOTS {
            let ability = definition[slot * TALENT_STRIDE + TALENT_ABILITY];
            let ultra = definition[slot * TALENT_STRIDE + TALENT_ULTRA] != 0;
            let highest = definition[slot * TALENT_STRIDE + TALENT_MAX_LEVEL].max(SINGLE_LEVEL);

            if ability == 0 {
                continue;
            }

            let level = match member.talents {
                Talents::None => continue,
                Talents::One(wanted, level) if wanted == ability => level.min(highest),
                Talents::One(..) => continue,
                Talents::Normal if ultra => continue,
                Talents::Normal | Talents::Every => highest,
            };

            levels.insert(ability, level);
        }
    }
}

pub fn stock_battle_items(ctx: &mut AppContext) -> Result<(), Fault> {
    for item in 0..BATTLE_ITEMS {
        let mut cell = [0u8; 8];

        cell[..4].copy_from_slice(&ITEM_STOCK.to_le_bytes());
        obfuscate_value(&mut cell);
        ctx.set_block_at(AppContext::ITEM_COUNTS_KIND_3 + item * 8, cell)?;
        ctx.set_block_at::<1>(AppContext::ITEMS_SELECTED + item, [1])?;
    }

    Ok(())
}
