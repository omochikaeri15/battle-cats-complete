use crate::Fault;

use super::{AppContext, Mamodel};

pub fn get_unit_model(ctx: &AppContext, faction: i32, button: i32) -> Result<Option<&Mamodel>, Fault> {
    if faction == 1 {
        return ctx.unit_models[1].get(button as i64 as usize).map(Some).ok_or(Fault::IndexOutOfRange {
            site: "get_unit_model",
            index: button as i64,
            limit: ctx.unit_models[1].len() as i64,
        });
    }

    if faction != 0 {
        return Ok(None);
    }

    ctx.unit_models[0].get(button as i64 as usize).map(Some).ok_or(Fault::IndexOutOfRange {
        site: "get_unit_model",
        index: button as i64,
        limit: ctx.unit_models[0].len() as i64,
    })
}
