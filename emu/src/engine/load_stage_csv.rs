use crate::Fault;

use super::{parse_stage_enemy_row, read_csv_cell, read_csv_row, AppContext, AssetStream, STAGE_ENEMY_COLUMNS};

pub fn load_stage_csv(ctx: &mut AppContext, stm: &mut AssetStream<'_>) -> Result<(), Fault> {
    read_csv_row(stm);

    ctx.set_i32_at(0x836fc, read_csv_cell(stm, 0) as i32)?;
    ctx.set_i32_at(0x46a9b4, 0)?;
    ctx.set_i32_at(0x46a9b4, read_csv_cell(stm, 1) as i32)?;
    ctx.set_i32_at(0x46a9b8, 0)?;
    ctx.set_i32_at(0x46a9b8, read_csv_cell(stm, 2) as i32)?;
    ctx.set_i32_at(0x46a9bc, 0)?;
    ctx.set_i32_at(0x46a9bc, read_csv_cell(stm, 3) as i32)?;
    ctx.set_i32_at(0x46a9c0, 0)?;
    ctx.set_i32_at(0x46a9c0, read_csv_cell(stm, 4) as i32)?;
    ctx.set_i32_at(0x46a9c4, 0)?;
    ctx.set_i32_at(0x46a9c4, read_csv_cell(stm, 5) as i32)?;

    read_csv_row(stm);

    ctx.set_i32_at(AppContext::STAGE_LENGTH, read_csv_cell(stm, 0) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x4, read_csv_cell(stm, 1) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x8, read_csv_cell(stm, 2) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0xc, read_csv_cell(stm, 3) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x10, read_csv_cell(stm, 4) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x14, read_csv_cell(stm, 5) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x18, read_csv_cell(stm, 6) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x1c, read_csv_cell(stm, 7) as i32)?;
    ctx.set_i32_at(AppContext::STAGE_LENGTH + 0x20, read_csv_cell(stm, 8) as i32)?;

    ctx.stage_enemies.clear();

    while read_csv_row(stm) {
        let mut entry = [0i32; STAGE_ENEMY_COLUMNS];
        parse_stage_enemy_row(&mut entry, stm);
        ctx.stage_enemies.push(entry);
    }

    Ok(())
}
