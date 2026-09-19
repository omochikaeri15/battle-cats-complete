use std::collections::BTreeMap;

use crate::{operation, Fault};

const SITE: &str = "get_cannon_effect";

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CannonGrowthStep {
    pub kind: i32,
    pub easing: i32,
    pub lv2: i32,
    pub value1: i32,
    pub value2: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CannonPart {
    pub growth: BTreeMap<i32, Vec<CannonGrowthStep>>,
}

pub fn get_cannon_effect(part: &mut CannonPart, effect: i32, level: i32) -> Result<i32, Fault> {
    let mut value = 0;

    if !part.growth.contains_key(&effect) {
        return Ok(value);
    }

    let mut from_level = 1i32;
    let mut step = 0usize;

    loop {
        let steps = part.growth.entry(effect).or_default();

        value = 0;

        if steps.len() <= step {
            break;
        }

        let progress = level.wrapping_sub(from_level);

        if level >= from_level {
            let out_of_range = Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: steps.len() as i64 };

            if part.growth.entry(effect).or_default().get(step).ok_or(out_of_range.clone())?.lv2 >= level {
                let easing = part.growth.entry(effect).or_default().get(step).ok_or(out_of_range.clone())?.easing;
                let lv2 = part.growth.entry(effect).or_default().get(step).ok_or(out_of_range.clone())?.lv2;
                let value1 = part.growth.entry(effect).or_default().get(step).ok_or(out_of_range.clone())?.value1;
                let value2 = part.growth.entry(effect).or_default().get(step).ok_or(out_of_range)?.value2;

                match easing {
                    2 => {
                        let span = value2.wrapping_sub(value1) as f32;
                        let ratio = progress as f32 / lv2.wrapping_sub(from_level) as f32;

                        value = operation::cvttss2si(value1 as f32 + (operation::powf(ratio + -1.0, 3.0) + 1.0) * span);
                    }
                    1 => {
                        let span = value2.wrapping_sub(value1) as f32;
                        let ratio = progress as f32 / lv2.wrapping_sub(from_level) as f32;

                        value = operation::cvttss2si(operation::powf(ratio, 3.0) * span + value1 as f32);
                    }
                    0 => {
                        let divisor = lv2.wrapping_sub(from_level);

                        value = operation::idiv(value2.wrapping_sub(value1).wrapping_mul(progress), divisor)
                            .ok_or(Fault::divide(SITE, divisor as i64))?
                            .wrapping_add(value1);
                    }
                    _ => value = 0,
                }

                break;
            }
        }

        from_level = part.growth.entry(effect).or_default().get(step).ok_or(Fault::IndexOutOfRange { site: SITE, index: step as i64, limit: 0 })?.lv2;
        step += 1;
    }

    Ok(value)
}
