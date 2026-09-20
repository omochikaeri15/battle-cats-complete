use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

use crate::{Fault, ops};

use super::{AppContext, call_rng, get_stage_index, labyrinth_load_floors};

#[derive(Clone, Default)]
pub struct LabyrinthFloor {
    pub min: i32,
    pub max: i32,
    pub results: Vec<i32>,
    pub weights: Vec<i32>,
}

pub fn labyrinth_roll_floor(ctx: &mut AppContext, mode: i32) -> Result<(), Fault> {
    labyrinth_load_floors(ctx)?;

    let key = match mode {
        1 => {
            let stage = get_stage_index(ctx)? as i64;

            ctx.i32_at((stage * 8 + 0x1c + AppContext::LABYRINTH as i64) as usize)?
        }
        0 => {
            let stage = get_stage_index(ctx)? as i64;

            ctx.i32_at((stage * 8 + 0x18 + AppContext::LABYRINTH as i64) as usize)?
        }
        _ => 0,
    };
    let min = ctx.labyrinth_floors.entry(key).or_default().min;
    let max = ctx.labyrinth_floors.entry(key).or_default().max;
    let min_again = ctx.labyrinth_floors.entry(key).or_default().min;
    let count = call_rng(ctx, max.wrapping_sub(min_again).wrapping_add(1)).wrapping_add(min);

    ctx.set_i32_at(AppContext::LABYRINTH_UNIT_COUNT, count)?;

    let roll = call_rng(ctx, 0x64);

    ctx.set_i32_at(AppContext::LABYRINTH_FLOOR_RESULT, 0)?;

    let mut sum = 0i32;
    let mut index = 0usize;

    while index < ctx.labyrinth_floors.entry(key).or_default().results.len() {
        let floor = ctx.labyrinth_floors.entry(key).or_default();
        let weight = *floor.weights.get(index).ok_or(Fault::index_out_of_range(index as i64, floor.weights.len() as i64))?;

        sum = sum.wrapping_add(weight);

        if roll < sum {
            let result = *ctx
                .labyrinth_floors
                .entry(key)
                .or_default()
                .results
                .get(index)
                .ok_or(Fault::index_out_of_range(index as i64, 0))?;

            ctx.set_i32_at(AppContext::LABYRINTH_FLOOR_RESULT, result)?;

            break;
        }

        index += 1;
    }

    let mut lineup: Vec<i32> = Vec::new();

    for slot in 0..10usize {
        let row = ctx.bytes_from(AppContext::BATTLE_LINEUP)?;
        let value = ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))?;

        lineup.push(value.wrapping_sub(2) as i32);
    }

    let mut index = 0usize;

    while index + 1 < lineup.len() {
        let span = (lineup.len() - 1 - index) as u64;
        let offset = (RandomState::new().build_hasher().finish() % (span + 1)) as usize;

        if offset != 0 {
            lineup.swap(index, index + offset);
        }

        index += 1;
    }

    ctx.labyrinth_units.clear();

    let count = ctx.i32_at(AppContext::LABYRINTH_UNIT_COUNT)?;
    let mut taken = 0i64;

    while taken < count as i64 {
        let unit = *lineup.get(taken as usize).ok_or(Fault::index_out_of_range(taken, 10))?;

        ctx.labyrinth_units.push(unit);
        taken += 1;
    }

    Ok(())
}
