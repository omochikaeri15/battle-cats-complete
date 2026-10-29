use super::STAGE_ENEMY_COLUMNS;

pub fn stage_entry_is_boss(enemy_row: &[i32; STAGE_ENEMY_COLUMNS]) -> bool {
    enemy_row[8] != 0
}
