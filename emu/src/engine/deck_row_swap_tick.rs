use crate::Fault;

use super::AppContext;

pub fn deck_row_swap_tick(ctx: &mut AppContext, target: i32) -> Result<(), Fault> {
    'swap: {
        if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
            ctx.set_i32_at(AppContext::DECK_ROW_SWAP_FRAME, 0)?;

            break 'swap;
        }

        let frame = ctx
            .i32_at(AppContext::DECK_ROW_SWAP_FRAME)?
            .wrapping_add(ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)?);

        ctx.set_i32_at(AppContext::DECK_ROW_SWAP_FRAME, frame)?;

        match target {
            1 => {
                if frame > 0 && ((frame.wrapping_sub(1)) as u32) <= 4 {
                    let offsets: [i32; 4] = match frame.wrapping_sub(1) {
                        0 | 4 => [20, -20, 0, 0],
                        1 | 3 => [40, -40, 0, 0],
                        _ => [80, -80, 0, 0],
                    };

                    for (index, offset) in offsets.iter().enumerate() {
                        ctx.set_i32_at(
                            AppContext::DECK_ROW_SWAP_OFFSETS.wrapping_add(index * 4),
                            *offset,
                        )?;
                    }

                    break 'swap;
                }

                ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [0])?;
                ctx.set_block_at::<16>(AppContext::DECK_ROW_SWAP_OFFSETS, [0; 16])?;
                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_FRAME, 0)?;

                match ctx.i32_at(AppContext::DECK_ROW_SHOWN)? {
                    1 => {
                        ctx.set_i32_at(AppContext::DECK_ROW_SHOWN, if frame > 0 { 0 } else { 1 })?
                    }
                    0 => {
                        ctx.set_i32_at(AppContext::DECK_ROW_SHOWN, if frame > 0 { 1 } else { 0 })?
                    }
                    _ => {}
                }
            }
            0 => {
                if frame > 0 && ((frame.wrapping_sub(1)) as u32) <= 4 {
                    let offsets: [i32; 4] = match frame.wrapping_sub(1) {
                        0 | 4 => [-20, 20, 0, 0],
                        1 | 3 => [-40, 40, 0, 0],
                        _ => [-80, 80, 0, 0],
                    };

                    for (index, offset) in offsets.iter().enumerate() {
                        ctx.set_i32_at(
                            AppContext::DECK_ROW_SWAP_OFFSETS.wrapping_add(index * 4),
                            *offset,
                        )?;
                    }

                    break 'swap;
                }

                ctx.set_block_at::<16>(AppContext::DECK_ROW_SWAP_OFFSETS, [0; 16])?;
                ctx.set_i32_at(AppContext::DECK_ROW_SWAP_FRAME, 0)?;

                let shown = ctx.i32_at(AppContext::DECK_ROW_SHOWN)?;

                if frame > 0 {
                    match shown {
                        0 => ctx.set_i32_at(AppContext::DECK_ROW_SHOWN, 1)?,
                        1 => ctx.set_i32_at(AppContext::DECK_ROW_SHOWN, 0)?,
                        _ => {}
                    }
                } else if (shown as u32) < 2 {
                    ctx.set_i32_at(AppContext::DECK_ROW_SHOWN, shown)?;
                }

                ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [0])?;
            }
            _ => {}
        }
    }

    for slot in 0..10usize {
        let counter = AppContext::faction_flags(0)
            .wrapping_add(AppContext::WALLET_SLOT_FLASH)
            .wrapping_add(slot * 4);
        let value = ctx.i32_at(counter)?;

        if value >= 0 {
            ctx.set_i32_at(
                counter,
                if (value as u32) < 9 {
                    value.wrapping_add(1)
                } else {
                    -1
                },
            )?;
        }
    }

    Ok(())
}
