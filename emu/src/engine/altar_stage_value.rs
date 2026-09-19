use super::AppContext;

pub fn altar_stage_value(ctx: &AppContext, map: i32, stage: i32) -> i32 {
    ctx.altar_stage_values
        .get(&map.wrapping_mul(100).wrapping_add(stage))
        .copied()
        .unwrap_or(0)
}
