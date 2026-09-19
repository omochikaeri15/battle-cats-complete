use super::STAGE_ENEMY_COLUMNS;

pub fn stage_entry_castle_defaults(enemy_row: &mut [i32; STAGE_ENEMY_COLUMNS]) {
    enemy_row[1] = 1;
    enemy_row[2] = 0;
    enemy_row[3] = 1;
    enemy_row[4] = 1;
    enemy_row[5] = 0;
}
