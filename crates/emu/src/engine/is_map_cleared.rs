use crate::Fault;

use super::{AppContext, get_stage_count, get_stages_cleared};

pub fn is_map_cleared(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    crown: i32,
    use_cache: u8,
) -> Result<bool, Fault> {
    match map_type.wrapping_add(0x19) as u32 {
        0x00 => Ok(ctx.u8_at(AppContext::MAP_NEG25_CLEARED)? != 0),
        0x0a => Ok(ctx.u8_at(AppContext::MAP_NEG15_CLEARED)? != 0),
        0x0b => {
            let moon = ctx
                .outbreak_cleared
                .entry(map_idx.wrapping_add(7))
                .or_default()
                .entry(0x2f)
                .or_default();

            Ok(*moon)
        }
        0x0c => {
            let moon = ctx
                .outbreak_cleared
                .entry(map_idx.wrapping_add(4))
                .or_default()
                .entry(0x2f)
                .or_default();

            Ok(*moon)
        }
        0x0d => {
            let moon = ctx
                .outbreak_cleared
                .entry(map_idx)
                .or_default()
                .entry(0x2f)
                .or_default();

            Ok(*moon)
        }
        _ => {
            let cleared = get_stages_cleared(ctx, map_type, map_idx, crown, use_cache as i32)?;
            let stage_count = get_stage_count(ctx, map_type, map_idx)?;

            Ok(cleared >= stage_count)
        }
    }
}
