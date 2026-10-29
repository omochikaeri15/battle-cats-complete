use crate::{Fault, ops};

use super::{AppContext, call_rng};

pub fn roll_drop_item_counts(
    ctx: &mut AppContext,
    map: i32,
    stage: i32,
    crown: i32,
) -> Result<Vec<i32>, Fault> {
    let mut counts = [0i32; 0x10];
    let record = ctx
        .drop_item_rows
        .values()
        .find(|record| record.map_id == map)
        .cloned();

    if let Some(record) = record {
        let mult = *record
            .crown_rates
            .get(crown as i64 as usize)
            .ok_or(Fault::index_out_of_range(crown as i64, record.crown_rates.len() as i64))?;
        let count =
            *record
                .stage_rates
                .get(stage as i64 as usize)
                .ok_or(Fault::index_out_of_range(stage as i64, record.stage_rates.len() as i64))?;
        let rolls = ops::cvttsd2si((mult * count) as f64 + 0.5);
        let mut roll = 0;

        while roll < rolls {
            if call_rng(ctx, 0x64) >= record.miss_weight {
                let total = record
                    .item_weights
                    .iter()
                    .fold(0i32, |sum, weight| sum.wrapping_add(*weight));
                let pick = call_rng(ctx, total);
                let mut sum = 0i32;

                for (index, weight) in record.item_weights.iter().enumerate() {
                    sum = sum.wrapping_add(*weight);

                    if pick < sum {
                        let slot = counts.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, 0x10))?;

                        *slot = slot.wrapping_add(1);

                        break;
                    }
                }
            }

            roll += 1;
        }
    }

    Ok(counts.to_vec())
}
