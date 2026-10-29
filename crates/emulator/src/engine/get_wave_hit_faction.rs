use crate::Fault;

use super::{AppContext, WaveRecord};

pub fn get_wave_hit_faction(ctx: &AppContext, wave_index: i32) -> Result<i32, Fault> {
    let record = AppContext::WAVE_RECORDS.wrapping_add(
        (wave_index as i64).wrapping_mul(AppContext::WAVE_RECORD_STRIDE as i64) as usize,
    );

    if ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? == 1 {
        return Ok(1);
    }

    Ok(((ctx.i32_at(record.wrapping_add(WaveRecord::IN_USE))? != 2) as i32).wrapping_neg())
}
