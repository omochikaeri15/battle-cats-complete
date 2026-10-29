use crate::Fault;

use super::{AppContext, Pinch};

pub fn touch_table_update(ctx: &mut AppContext, kind: i32, x: i32, y: i32, id: i32) -> Result<(), Fault> {
    let table = AppContext::PINCH;

    if kind == 0 {
        let mut first_free = 1u8;
        let slot: usize;

        'pick: {
            if ctx.u8_at(table + Pinch::FIRST_DOWN)? != 0 {
                if ctx.i32_at(table + Pinch::FIRST_ID)? != id {
                    first_free = 0;

                    if ctx.u8_at(table + Pinch::SECOND_DOWN)? == 0 {
                        slot = (first_free ^ 1) as usize;
                        break 'pick;
                    }
                } else {
                    ctx.set_block_at::<1>(table + Pinch::FIRST_DOWN, [0])?;

                    if ctx.u8_at(table + Pinch::SECOND_DOWN)? == 0 {
                        slot = (first_free ^ 1) as usize;
                        break 'pick;
                    }
                }
            } else if ctx.u8_at(table + Pinch::SECOND_DOWN)? == 0 {
                slot = (first_free ^ 1) as usize;
                break 'pick;
            }

            if ctx.i32_at(table + Pinch::SECOND_ID)? == id {
                ctx.set_block_at::<1>(table + Pinch::SECOND_DOWN, [0])?;
                slot = (first_free ^ 1) as usize;
                break 'pick;
            }

            if first_free != 0 {
                slot = 0;
                break 'pick;
            }

            return Ok(());
        }

        ctx.set_block_at::<1>(table + Pinch::FIRST_DOWN + slot, [1])?;
        ctx.set_i32_at(table + Pinch::FIRST_X + slot * 4, x)?;
        ctx.set_i32_at(table + Pinch::FIRST_Y + slot * 4, y)?;
        ctx.set_i32_at(table + Pinch::FIRST_ID + slot * 4, id)?;
    } else if kind == 1 {
        if ctx.u8_at(table + Pinch::FIRST_DOWN)? != 0 && ctx.i32_at(table + Pinch::FIRST_ID)? == id {
            ctx.set_i32_at(table + Pinch::FIRST_X, x)?;
            ctx.set_i32_at(table + Pinch::FIRST_Y, y)?;
        } else if ctx.u8_at(table + Pinch::SECOND_DOWN)? != 0 && ctx.i32_at(table + Pinch::SECOND_ID)? == id {
            ctx.set_i32_at(table + Pinch::SECOND_X, x)?;
            ctx.set_i32_at(table + Pinch::SECOND_Y, y)?;
        }
    } else if kind == 2 {
        if ctx.u8_at(table + Pinch::FIRST_DOWN)? != 0 && ctx.i32_at(table + Pinch::FIRST_ID)? == id {
            ctx.set_block_at::<1>(table + Pinch::FIRST_DOWN, [0])?;
        }

        if ctx.u8_at(table + Pinch::SECOND_DOWN)? != 0 && ctx.i32_at(table + Pinch::SECOND_ID)? == id {
            ctx.set_block_at::<1>(table + Pinch::SECOND_DOWN, [0])?;
        }
    }

    Ok(())
}
