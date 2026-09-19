use super::STAGE_ENEMY_COLUMNS;

pub fn stage_enemy_valid(enemy_row: &[i32; STAGE_ENEMY_COLUMNS]) -> bool {
    enemy_row[0] != 0
}
