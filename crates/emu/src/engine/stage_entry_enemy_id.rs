use super::STAGE_ENEMY_COLUMNS;

pub fn stage_entry_enemy_id(enemy_row: &[i32; STAGE_ENEMY_COLUMNS]) -> i32 {
    enemy_row[0].wrapping_add(-2)
}
