use emu::engine::{AppContext, SLOTS_PER_FACTION};

const CAT_FACTION: i32 = 0;
const ENEMY_FACTION: i32 = 1;
const BASE_OCCUPANT: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Occupant {
    Base,
    Unit { id: i32, name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Vitals {
    pub(crate) occupant: Occupant,
    pub(crate) hp: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Diagnostics {
    pub(crate) rng: u32,
    pub(crate) cats: Vec<Vitals>,
    pub(crate) enemies: Vec<Vitals>,
}

pub(super) fn read(ctx: &AppContext) -> Diagnostics {
    Diagnostics { rng: ctx.rng_state(), cats: side(ctx, CAT_FACTION), enemies: side(ctx, ENEMY_FACTION) }
}

fn side(ctx: &AppContext, faction: i32) -> Vec<Vitals> {
    (0..SLOTS_PER_FACTION)
        .filter_map(|slot| {
            let raw = emu::engine::get_occupant(ctx, faction, slot).ok().filter(|raw| *raw != 0)?;
            let occupant = if raw == BASE_OCCUPANT {
                Occupant::Base
            } else {
                let id = emu::engine::get_slot_unit_id(ctx, faction, slot).ok()?;
                let name = emu::engine::get_unit_name(ctx, faction, slot).ok().unwrap_or_default();

                Occupant::Unit { id, name: String::from_utf8_lossy(&name).trim().to_owned() }
            };

            Some(Vitals { occupant, hp: emu::engine::get_hp(ctx, faction, slot).ok()? })
        })
        .collect()
}
