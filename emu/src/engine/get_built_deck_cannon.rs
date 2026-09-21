use crate::Fault;

use super::{AppContext, find_built_deck_4crown, has_built_deck};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct BuiltDeckRecord {
    pub rows: [(i16, i8); 10],
    pub cannon_parts: u16,
    pub cannon_type: u8,
}

pub fn get_built_deck_cannon(ctx: &mut AppContext, stage_key: i32) -> Result<i32, Fault> {
    if !has_built_deck(ctx, stage_key)? {
        return Ok(0);
    }

    let mut deck_id = -1i32;

    for (candidate, stage_keys) in ctx.built_deck_stages.iter() {
        for listed in stage_keys {
            if *listed == stage_key {
                deck_id = *candidate as i32;
            }
        }
    }

    if stage_key < 1_000_000_000 {
        if deck_id == -1 {
            return Ok(0);
        }
    } else if deck_id == -1 {
        deck_id = find_built_deck_4crown(ctx, stage_key);

        if deck_id == -1 {
            return Ok(0);
        }
    }

    let record = ctx
        .built_deck_records
        .get(&(deck_id as i16))
        .ok_or(Fault::key_not_found(deck_id as i64))?;

    Ok(((record.cannon_type as u32) << 0x10 | record.cannon_parts as u32) as i32)
}
