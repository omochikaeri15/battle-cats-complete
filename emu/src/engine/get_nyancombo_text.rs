use crate::Fault;

use super::{format_string2, get_charagroup_text, AppContext, NyancomboRecord};

pub fn get_nyancombo_text(ctx: &mut AppContext, record: &NyancomboRecord, line: i32) -> Result<Vec<u8>, Fault> {
    match line {
        0 => Ok(ctx.combo_names.get(record.name_index as i64 as usize).cloned().unwrap_or_default()),
        1 => {
            let effect = ctx.combo_effect_texts.get(record.kind[0] as i64 as usize).cloned().unwrap_or_default();
            let power = ctx.combo_power_texts.get(record.power[0] as i64 as usize).cloned().unwrap_or_default();

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
