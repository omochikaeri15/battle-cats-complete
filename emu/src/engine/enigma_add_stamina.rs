use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, medal_awarded};

#[derive(Clone, Default)]
pub struct EnigmaGroup {
    pub id: i32,
    pub map_stage: i32,
    pub stage_ids: Vec<i32>,
    pub weights: Vec<i32>,
}

#[derive(Clone, Default)]
pub struct EnigmaTier {
    pub thresholds: Vec<i32>,
    pub chances: Vec<i32>,
}

#[derive(Clone, Default)]
pub struct EnigmaPool {
    pub groups: Vec<i32>,
    pub weights: Vec<i32>,
}

#[derive(Clone, Default)]
pub struct EnigmaEntry {
    pub groups: Vec<i32>,
    pub chances: Vec<i32>,
}

#[derive(Clone, Copy, Default)]
pub struct EnigmaActive {
    pub state: i32,
    pub time: f64,
    pub group: i32,
    pub map: i32,
}

#[derive(Default)]
pub struct Enigma {
    pub tiers: Vec<EnigmaTier>,
    pub pools: Vec<EnigmaPool>,
    pub table: BTreeMap<i32, BTreeMap<i32, EnigmaEntry>>,
    pub groups: Vec<EnigmaGroup>,
    pub medals: Vec<i32>,
    pub active: Vec<EnigmaActive>,
    pub stamina: i32,
    pub last_threshold: i32,
    pub medal_count: i32,
    pub pending_set: u8,
    pub pending: EnigmaActive,
}

pub fn enigma_add_stamina(ctx: &mut AppContext, amount: i32) -> Result<(), Fault> {
    if ctx.enigma.medals.is_empty() {
        return Ok(());
    }

    let mut owned = 0i32;
    let mut index = 0usize;

    while index < ctx.enigma.medals.len() {
        let medal = *ctx.enigma.medals.get(index).ok_or(Fault::index_out_of_range(index as i64, 0))?;

        owned = owned.wrapping_add(medal_awarded(ctx, medal) as i32);
        index += 1;
    }

    if owned == 0 {
        return Ok(());
    }

    ctx.enigma.stamina = amount.wrapping_add(ctx.enigma.stamina);

    if ctx.enigma.stamina >= 0x1f41 {
        ctx.enigma.stamina = 0;
        ctx.enigma.last_threshold = 0;
    }

    Ok(())
}
