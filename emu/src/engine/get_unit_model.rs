use crate::Fault;

use super::AppContext;

pub fn get_unit_model(ctx: &AppContext, faction: i32, button: i32) -> Result<Option<(usize, usize)>, Fault> {
    if faction == 1 {
        if button as i64 as usize >= ctx.unit_models[1].len() {
            return Err(Fault::IndexOutOfRange { site: "get_unit_model", index: button as i64, limit: ctx.unit_models[1].len() as i64 });
        }

        return Ok(Some((1, button as i64 as usize)));
    }

    if faction != 0 {
        return Ok(None);
    }

    if button as i64 as usize >= ctx.unit_models[0].len() {
        return Err(Fault::IndexOutOfRange { site: "get_unit_model", index: button as i64, limit: ctx.unit_models[0].len() as i64 });
    }

    Ok(Some((0, button as i64 as usize)))
}
