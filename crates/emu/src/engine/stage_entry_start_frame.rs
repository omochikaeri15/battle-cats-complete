use super::STAGE_ENEMY_COLUMNS;

pub fn stage_entry_start_frame(enemy_row: &[i32; STAGE_ENEMY_COLUMNS]) -> i32 {
    enemy_row[2]
}
