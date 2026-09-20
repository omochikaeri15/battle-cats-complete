use crate::Fault;

use super::{AppContext, OrbDef, format_string2};

pub fn orb_name(ctx: &mut AppContext, def: &OrbDef) -> Result<Vec<u8>, Fault> {
    let pattern = ctx
        .orb_store
        .orb_explanations
        .get(def.abil as i64 as usize)
        .ok_or(Fault::out_of_range())?[0]
        .clone();
    let grade = ctx
        .orb_store
        .grades
        .get(def.grade as i64 as usize)
        .ok_or(Fault::out_of_range())?
        .name
        .clone();
    let attribute = if def.trait_index as i64 != 0xc {
        ctx.orb_store
            .trait_explanations
            .get(def.trait_index as i64 as usize)
            .ok_or(Fault::out_of_range())?[0]
            .clone()
    } else {
        Vec::new()
    };

    format_string2(ctx, &pattern, &grade, &attribute)
}
