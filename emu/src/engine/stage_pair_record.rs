use crate::Fault;

use super::AppContext;

#[derive(Clone, Default)]
pub struct StagePairRecord {
    pub other_stage: i32,
    pub message: Vec<u8>,
}

pub fn stage_pair_record(
    ctx: &AppContext,
    map: i32,
    stage: i32,
) -> Result<Option<StagePairRecord>, Fault> {
    let key = map.wrapping_mul(100).wrapping_add(stage);

    if ctx.stage_pair_records.is_empty() || !ctx.stage_pair_records.contains_key(&key) {
        return Ok(None);
    }

    Ok(Some(
        ctx.stage_pair_records
            .get(&key)
            .ok_or(Fault::key_not_found(key as i64))?
            .clone(),
    ))
}
