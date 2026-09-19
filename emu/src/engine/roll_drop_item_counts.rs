use crate::{Fault, operation};

use super::{AppContext, call_rng};

const SITE: &str = "roll_drop_item_counts";

#[derive(Clone, Default)]
pub struct DropRecord {
    pub map_id: i32,
    pub star_mults: Vec<f32>,
    pub stage_counts: Vec<f32>,
    pub miss: i32,
    pub weights: Vec<i32>,
}

pub fn roll_drop_item_counts(
    ctx: &mut AppContext,
    map: i32,
    stage: i32,
    star: i32,
) -> Result<Vec<i32>, Fault> {
    let mut counts = [0i32; 0x10];
    let record = ctx
        .drop_items
        .values()
        .find(|record| record.map_id == map)
        .cloned();

    if let Some(record) = record {
        let mult = *record
            .star_mults
            .get(star as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: star as i64,
                limit: record.star_mults.len() as i64,
            })?;
        let count =
            *record
                .stage_counts
                .get(stage as i64 as usize)
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: stage as i64,
                    limit: record.stage_counts.len() as i64,
                })?;
        let rolls = operation::cvttsd2si((mult * count) as f64 + 0.5);
        let mut roll = 0;

        while roll < rolls {
            if call_rng(ctx, 0x64) >= record.miss {
                let total = record
                    .weights
                    .iter()
                    .fold(0i32, |sum, weight| sum.wrapping_add(*weight));
                let pick = call_rng(ctx, total);
                let mut sum = 0i32;

                for (index, weight) in record.weights.iter().enumerate() {
                    sum = sum.wrapping_add(*weight);

                    if pick < sum {
                        let slot = counts.get_mut(index).ok_or(Fault::IndexOutOfRange {
                            site: SITE,
                            index: index as i64,
                            limit: 0x10,
                        })?;

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
