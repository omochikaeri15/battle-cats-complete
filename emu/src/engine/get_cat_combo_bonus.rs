use crate::Fault;

use super::{charagroup_has_unit, AppContext};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct NyancomboRecord {
    pub combo_id: i32,
    pub unit_count: i32,
    pub unlock_gate: i32,
    pub charagroup_id: i32,
    pub name_index: i32,
    pub kind: [i32; 3],
    pub power: [i32; 3],
    pub effect_count: i32,
    pub banner_pending: u8,
    pub unit: [i32; 5],
    pub form: [i32; 5],
    pub enabled: u8,
    pub state: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ComboStore {
    pub records: Vec<NyancomboRecord>,
    pub params: Vec<Vec<i32>>,
}

pub fn get_cat_combo_bonus(ctx: &AppContext, table: &ComboStore, kind: i32, unit_id: i32) -> Result<i32, Fault> {
    let mut bonus = 0i32;

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
                site: "get_cat_combo_bonus",
                index: effect,
                limit: effects.len() as i64,
            })?;

            if effect_kind == kind
                && (record.charagroup_id == -1 || charagroup_has_unit(&ctx.chara_groups, record.charagroup_id, unit_id))
            {
                let power = *effects.get((effect + 3) as usize).ok_or(Fault::IndexOutOfRange {
                    site: "get_cat_combo_bonus",
                    index: effect,
                    limit: 4,
                })?;
                let value = table
                    .params
                    .get(kind as usize)
                    .and_then(|powers| powers.get(power as usize))
                    .ok_or(Fault::IndexOutOfRange { site: "get_cat_combo_bonus", index: power as i64, limit: 0 })?;

                bonus = bonus.wrapping_add(*value);
            }

            effect += 1;

            if effect >= record.effect_count as i64 {
                break;
            }
        }
    }

    Ok(bonus)
}
