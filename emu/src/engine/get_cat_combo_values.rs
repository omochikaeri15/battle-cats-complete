use crate::Fault;

use super::{charagroup_has_unit, AppContext, ComboStore};

pub fn get_cat_combo_values(ctx: &AppContext, table: &ComboStore, kind: i32, unit_id: i32) -> Result<Vec<i32>, Fault> {
    let mut out = Vec::new();

    for record in &table.records {
        if record.enabled == 0 || record.effect_count <= 0 {
            continue;
        }

        let effects = [
            record.kind[0],
            record.kind[1],
            record.kind[2],
            record.power[0],
            record.power[1],
            record.power[2],
            record.effect_count,
        ];
        let mut effect = 0i64;

        loop {
            let effect_kind = *effects.get(effect as usize).ok_or(Fault::IndexOutOfRange {
                site: "get_cat_combo_values",
                index: effect,
                limit: effects.len() as i64,
            })?;

            if effect_kind == kind
                && (record.charagroup_id == -1 || charagroup_has_unit(&ctx.chara_groups, record.charagroup_id, unit_id))
            {
                let power = *effects.get((effect + 3) as usize).ok_or(Fault::IndexOutOfRange {
                    site: "get_cat_combo_values",
                    index: effect,
                    limit: 4,
                })?;
                let value = table
                    .params
                    .get(kind as usize)
                    .and_then(|powers| powers.get(power as usize))
                    .ok_or(Fault::IndexOutOfRange { site: "get_cat_combo_values", index: power as i64, limit: 0 })?;

                out.push(*value);
            }

            effect += 1;

            if effect >= record.effect_count as i64 {
                break;
            }
        }
    }

    Ok(out)
}
