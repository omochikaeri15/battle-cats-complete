use crate::Fault;

use super::{AppContext, config_json_int, treasure_area_match, xor_row46_get};

const CHAPTER_STAMINA_BONUS: [i32; 10] = [0, 10, 20, 10, 0, 0, 0, 0, 0, 0];

pub fn stage_stamina_cost(ctx: &mut AppContext, stage: i32, halve: u8) -> Result<i32, Fault> {
    let row = (AppContext::MAP_STAGE_ROWS as i64
        + (stage as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64) as usize;
    let base = xor_row46_get(ctx.bytes_from(row)?, 0x18).ok_or(Fault::index_out_of_range(0x18, 0x2e))? as i32;
    let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
    let mut cost = base;

    if chapter != 3 {
        let bonus =
            *CHAPTER_STAMINA_BONUS
                .get(chapter as i64 as usize)
                .ok_or(Fault::index_out_of_range(chapter as i64, 10))?;

        cost = base.wrapping_add(bonus);

        let area = config_json_int(ctx, b"CnfJsonUseStaminaDown", b"area", 1, 9)?;

        if treasure_area_match(chapter, area) {
            let percent = config_json_int(ctx, b"CnfJsonUseStaminaDown", b"value", 0x1e, 0x64)?;

            cost = percent.wrapping_mul(cost) / 0x64;
        }
    }

    if halve == 0 {
        return Ok(cost);
    }

    Ok(cost / 2)
}
