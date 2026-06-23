// TODO: Fill these manual XP values in from like the wiki
pub fn get_hardcoded_xp(global_map_id: u32, stage_id: usize) -> u32 {
    if stage_id >= 48 {
        return 0;
    }

    match global_map_id {
        3000 | 3001 | 3002 => {
            const EOC_XP: [u32; 48] = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            EOC_XP[stage_id]
        }

        3003 | 3004 | 3005 => {
            const ITF_XP: [u32; 48] = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            ITF_XP[stage_id]
        }

        3006 | 3007 | 3008 => {
            const COTC_XP: [u32; 48] = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            COTC_XP[stage_id]
        }

        _ => 0,
    }
}