use crate::Fault;

use super::{std_map_int_maanim_subscript, AppContext, Maanim};

pub fn get_unit_anim(ctx: &mut AppContext, faction: i32, button: i32, key: i32) -> Result<Option<&mut Maanim>, Fault> {
    if faction == 1 {
        let limit = ctx.unit_anims[1].len() as i64;
        let anims = ctx.unit_anims[1].get_mut(button as i64 as usize).ok_or(Fault::IndexOutOfRange {
            site: "get_unit_anim",
            index: button as i64,
            limit,
        })?;

        return Ok(Some(std_map_int_maanim_subscript(anims, &key)));
    }

    if faction != 0 {
        return Ok(None);
    }

    let limit = ctx.unit_anims[0].len() as i64;
    let anims = ctx.unit_anims[0].get_mut(button as i64 as usize).ok_or(Fault::IndexOutOfRange {
        site: "get_unit_anim",
        index: button as i64,
        limit,
    })?;

    Ok(Some(std_map_int_maanim_subscript(anims, &key)))
}
