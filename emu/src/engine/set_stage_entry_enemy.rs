use super::STAGE_ENEMY_COLUMNS;

pub fn set_stage_entry_enemy(enemy_row: &mut [i32; STAGE_ENEMY_COLUMNS], enemy_id: i32) {
    enemy_row[0] = enemy_id.wrapping_add(2);
}
