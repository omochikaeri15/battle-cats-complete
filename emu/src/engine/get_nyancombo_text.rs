use crate::Fault;

use super::{AppContext, NyancomboRecord, format_string2, get_charagroup_text};

pub fn get_nyancombo_text(
    ctx: &mut AppContext,
    record: &NyancomboRecord,
    line: i32,
) -> Result<Vec<u8>, Fault> {
    match line {
        0 => Ok(ctx
            .combo_names
            .get(record.name_index as i64 as usize)
            .cloned()
            .ok_or(Fault::index_out_of_range(record.name_index as i64, ctx.combo_names.len() as i64))?),
        1 => {
            let effect = ctx
                .combo_effect_texts
                .get(record.kind[0] as i64 as usize)
                .cloned()
                .ok_or(Fault::index_out_of_range(record.kind[0] as i64, ctx.combo_effect_texts.len() as i64))?;
            let power = ctx
                .combo_power_texts
                .get(record.power[0] as i64 as usize)
                .cloned()
                .ok_or(Fault::index_out_of_range(record.power[0] as i64, ctx.combo_power_texts.len() as i64))?;

            format_string2(ctx, b"%@%@", &effect, &power)
        }
        _ => {
            if record.charagroup_id == -1 {
                return Ok(Vec::new());
            }

            Ok(get_charagroup_text(ctx, record.charagroup_id))
        }
    }
}
