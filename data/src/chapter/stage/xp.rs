//! Experience rewards the engine hardcodes rather than storing in data files.

/// Returns the experience reward for a stage whose value the engine hardcodes.
///
/// The three main story chapters omit these values from their stage metadata, so
/// they are reproduced here. Other maps carry theirs in their own metadata and
/// yield zero.
///
/// # Arguments
/// * `global_map_id` - The map's identifier in the global numbering scheme.
/// * `stage_id` - The index of the stage within its map.
///
/// # Returns
/// A `u32` containing the hardcoded experience reward, or zero when the map does
/// not hardcode one or the stage index lies beyond the chapter's length.
pub fn get_hardcoded_xp(global_map_id: u32, stage_id: usize) -> u32 {
    if stage_id >= 48 {
        return 0;
    }

    match global_map_id {
        3000..=3002 => {
            const EOC_XP: [u32; 48] = [
                1000, 1300, 1600, 1900, 2200, 2500, 2800, 2800,
                3400, 3700, 4000, 4300, 4600, 4900, 5200, 5500,
                5800, 6100, 6400, 6400, 7000, 7300, 7600, 7900,
                8200, 8500, 8800, 9100, 9400, 9700, 10000, 10300,
                10600, 10600, 11200, 11500, 11800, 12100, 12400, 12700,
                13000, 13300, 13600, 13900, 14200, 14500, 14800, 15100,
            ];
            EOC_XP[stage_id]
        }

        3003..=3005 => {
            const ITF_XP: [u32; 48] = [
                1000, 1300, 1600, 1900, 2200, 2500, 2800, 3100,
                3400, 3700, 4000, 4300, 4600, 4900, 5200, 5500,
                5800, 5500, 6400, 6700, 7000, 7300, 7600, 7900,
                8200, 7300, 8800, 9100, 9400, 9700, 10000, 10300,
                10600, 10900, 11200, 11500, 11800, 11500, 12400, 12700,
                13000, 13300, 13600, 13900, 13900, 13900, 14200, 14500,
            ];
            ITF_XP[stage_id]
        }

        3006..=3008 => {
            const COTC_XP: [u32; 48] = [
                1000, 1300, 1600, 1900, 2200, 2500, 2800, 3100,
                3400, 3700, 4000, 4300, 4600, 4900, 5200, 5500,
                5800, 6100, 6400, 6700, 7000, 7300, 7600, 7900,
                8200, 8500, 8800, 9700, 9400, 9700, 10000, 10300,
                10600, 10900, 11200, 11200, 11800, 12100, 12400, 12700,
                13000, 13300, 13600, 13900, 13900, 13900, 14200, 14500,
            ];
            COTC_XP[stage_id]
        }

        _ => 0,
    }
}