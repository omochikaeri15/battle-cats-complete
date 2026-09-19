use crate::Fault;

use super::{AppContext, lose_exit_map_check};

const SITE: &str = "lose_tip_allowed";

pub fn lose_tip_allowed(ctx: &mut AppContext, id: i32) -> Result<bool, Fault> {
    let mut cells: Vec<i32> = Vec::new();
    let mut row = -1i64;

    loop {
        row += 1;

        if row >= ctx.lose_text_settings.len() as i32 as i64 {
            break;
        }

        let first = *ctx.lose_text_settings[row as usize]
            .first()
            .ok_or(Fault::NullPointer { site: SITE })?;

        if first == id {
            cells = ctx.lose_text_settings[row as usize].clone();

            break;
        }
    }

    let mut column = 1usize;
    let mut reached = false;

    loop {
        let cell = *cells.get(column).ok_or(if cells.is_empty() {
            Fault::NullPointer { site: SITE }
        } else {
            Fault::IndexOutOfRange {
                site: SITE,
                index: column as i64,
                limit: cells.len() as i64,
            }
        })?;

        if cell == 0 && (column as u32).wrapping_sub(1) <= 0x19 {
            let mode = ctx.i32_at(AppContext::CHAPTER_MODE)?;

            let rejected = match column {
                1 => mode == 0 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                2 => mode == 0 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                3 => mode == 1 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                4 => mode == 1 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                5 => mode == 2 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                6 => mode == 2 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                7 => lose_exit_map_check(ctx)?,
                8 => mode == 4 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                9 => mode == 4 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                10 => mode == 5 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                11 => mode == 5 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                12 => mode == 6 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                13 => mode == 6 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                14 => mode == 7 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                15 => mode == 7 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                16 => mode == 8 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                17 => mode == 8 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                18 => mode == 9 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? == 0,
                19 => mode == 9 && ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0,
                20 => mode == 9 && ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0,
                21 => {
                    mode == 3
                        && ctx.i32_at(AppContext::STAR_LEVEL)? == 0
                        && !lose_exit_map_check(ctx)?
                }
                22 => {
                    mode == 3
                        && ctx.i32_at(AppContext::STAR_LEVEL)? == 1
                        && !lose_exit_map_check(ctx)?
                }
                23 => {
                    mode == 3
                        && ctx.i32_at(AppContext::STAR_LEVEL)? == 2
                        && !lose_exit_map_check(ctx)?
                }
                24 => {
                    mode == 3
                        && ctx.i32_at(AppContext::STAR_LEVEL)? == 3
                        && !lose_exit_map_check(ctx)?
                }
                25 => {
                    mode == 3
                        && ctx.i32_at(AppContext::STAR_LEVEL)? == 4
                        && !lose_exit_map_check(ctx)?
                }
                _ => mode == 0x63 && !lose_exit_map_check(ctx)?,
            };

            if rejected {
                return Ok(reached);
            }
        }

        reached = column >= 0x1a;
        column += 1;

        if column == 0x1b {
            return Ok(reached);
        }
    }
}
