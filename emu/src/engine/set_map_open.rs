use crate::Fault;

use super::AppContext;

pub fn set_map_open(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    star: i32,
    value: i32,
) -> Result<(), Fault> {
    if map_type as u32 <= 4 {
        let cell = (map_type as u32 as i64) * 0x7d0
            + (map_idx as i64) * 4
            + star as i64
            + AppContext::MAP_OPEN_STORY as i64;

        return ctx.set_block_at::<1>(cell as usize, [value as u8]);
    }

    let case = map_type.wrapping_add(0x1a) as u32;

    match case {
        0x00 | 0x02..=0x04 => {
            let maps = match case {
                0x00 => &mut ctx.map_open_neg26,
                0x02 => &mut ctx.map_open_neg24,
                0x03 => &mut ctx.map_open_neg23,
                _ => &mut ctx.map_open_neg22,
            };
            let limit = maps.len() as i64;
            let cell = maps
                .get_mut(map_idx as i64 as usize)
                .and_then(|stars| stars.get_mut(star as i64 as usize))
                .ok_or(Fault::index_out_of_range(map_idx as i64, limit))?;

            *cell = value as i8;

            Ok(())
        }
        0x06 | 0x08..=0x0a | 0x0f => {
            let (maps, cell) = match case {
                0x06 => (&mut ctx.map_open_neg20, (map_idx as i64) * 4 + star as i64),
                0x08 => (&mut ctx.map_open_neg18, (map_idx as i64) * 4 + star as i64),
                0x09 => (&mut ctx.map_open_neg17, map_idx as i64 + star as i64),
                0x0a => (&mut ctx.map_open_neg16, (map_idx as i64) * 4 + star as i64),
                _ => (&mut ctx.map_open_neg11, (map_idx as i64) * 4 + star as i64),
            };
            let limit = maps.len() as i64;
            let slot = maps.get_mut(cell as usize).ok_or(Fault::index_out_of_range(cell, limit))?;

            *slot = value as i8;

            Ok(())
        }
        0x10 | 0x11 | 0x16 => {
            let maps = match case {
                0x10 => &mut ctx.map_open_neg10,
                0x11 => &mut ctx.map_open_neg9,
                _ => &mut ctx.map_open_neg4,
            };
            let cell = (map_idx as i64) * 4 + star as i64;
            let limit = maps.len() as i64;
            let slot = maps.get_mut(cell as usize).ok_or(Fault::index_out_of_range(cell, limit))?;

            *slot = value;

            Ok(())
        }
        0x14 => ctx.set_i32_at(
            ((map_idx as i64) * 0x10 + (star as i64) * 4 + AppContext::MAP_OPEN_NEG6 as i64)
                as usize,
            value,
        ),
        _ => Ok(()),
    }
}
