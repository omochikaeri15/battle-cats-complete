use super::AppContext;

pub fn altar_replacement(ctx: &AppContext, enemy_id: i32) -> i32 {
    for (enemy, replacement) in &ctx.altar_enemy_ids {
        if *enemy == enemy_id {
            return *replacement;
        }
    }

    -1
}
