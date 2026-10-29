use crate::Fault;

use super::{AppContext, find_built_deck_4crown, has_built_deck};

pub fn get_built_deck_rows(ctx: &mut AppContext, stage_key: i32) -> Result<[(i16, i8); 10], Fault> {
    if !has_built_deck(ctx, stage_key)? {
        return Ok([(-1, 0); 10]);
    }

    let mut deck_id = -1i32;

    for (candidate, stage_keys) in ctx.built_deck_stages.iter() {
        for listed in stage_keys {
            if *listed == stage_key {
                deck_id = *candidate as i32;
            }
        }
    }

    if stage_key >= 1_000_000_000 && deck_id == -1 {
        deck_id = find_built_deck_4crown(ctx, stage_key);
    }

    if deck_id == -1 {
        return Ok([(-1, 0); 10]);
    }

    Ok(ctx
        .built_deck_records
        .get(&(deck_id as i16))
        .ok_or(Fault::key_not_found(deck_id as i64))?
        .rows)
}
