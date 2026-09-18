use super::STAGE_ENEMY_COLUMNS;

pub fn stage_entry_atk_mag(enemy_row: &[i32; STAGE_ENEMY_COLUMNS]) -> i32 {
    if enemy_row[11] != 0 {
        return enemy_row[11];
    }

    enemy_row[9]
}
