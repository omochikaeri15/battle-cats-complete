use crate::Fault;

use super::{AppContext, get_condition_flag, max_i32};

#[derive(Default)]
pub struct ReleasePoint {
    pub conditions: Vec<i32>,
    pub caps: Vec<i32>,
    pub popup: Vec<u8>,
    pub headline_label: Vec<u8>,
    pub explanation_label: Vec<u8>,
}

pub fn get_release_point_cap(ctx: &AppContext, point_id: i32) -> Result<i32, Fault> {
    let Some(release) = ctx.release_points.get(&point_id) else {
        return Ok(0);
    };
    let mut cap = 0;
    let mut index = 0usize;

    while index < release.conditions.len() {
        let condition = *release
            .conditions
            .get(index)
            .ok_or(Fault::index_out_of_range(index as i64, release.conditions.len() as i64))?;

        if get_condition_flag(&ctx.server_flags, condition).is_some_and(|flag| flag) {
            cap = max_i32(
                cap,
                *release.caps.get(index).ok_or(Fault::index_out_of_range(index as i64, release.caps.len() as i64))?,
            );
        }

        index += 1;
    }

    Ok(cap)
}
