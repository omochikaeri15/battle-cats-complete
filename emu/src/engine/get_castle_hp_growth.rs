use crate::{operation, Fault};

use super::{get_cannon_base_level, AppContext};

const SITE: &str = "get_castle_hp_growth";

pub fn get_castle_hp_growth(ctx: &AppContext, level: i32) -> Result<i32, Fault> {
    let level = if level == -1 { get_cannon_base_level(ctx)?.wrapping_add(1) } else { level };
    let mut from_level = 1i32;

    for step in &ctx.castle_hp_growth {
        let progress = level.wrapping_sub(from_level);

        if progress >= 0 && level <= step.lv2 {
            let span = step.value2.wrapping_sub(step.value1);
            let divisor = step.lv2.wrapping_sub(from_level);

            return match step.easing {
                2 => {
                    let ratio = progress as f32 / divisor as f32;

                    Ok(operation::cvttss2si(step.value1 as f32 + (operation::powf(ratio + -1.0, 3.0) + 1.0) * span as f32))
                }
                1 => {
                    let ratio = progress as f32 / divisor as f32;

                    Ok(operation::cvttss2si(operation::powf(ratio, 3.0) * span as f32 + step.value1 as f32))
                }
                0 => Ok(operation::idiv(span.wrapping_mul(progress), divisor).ok_or(Fault::divide(SITE, divisor as i64))?.wrapping_add(step.value1)),
                _ => Ok(0),
            };
        }

        from_level = step.lv2;
    }

    Ok(0)
}
