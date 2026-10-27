use crate::Fault;

use super::SoundManager;

pub fn set_se_volume_setting(mgr: &mut SoundManager, percent: i32) -> Result<(), Fault> {
    mgr.se_volume = percent;

    let mut index = 0i64;

    while index < mgr.count as i64 {
        let limit = mgr.kinds.len() as i64;

        if *mgr.kinds.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))? & 2 != 0 {
            let limit = mgr.sources.len() as i64;

            if let Some(source) = mgr.sources.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))? {
                let limit = mgr.volumes.len() as i64;
                let level = *mgr.volumes.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))?;

                source.set_volume(level.wrapping_mul(mgr.se_volume) as f32 / 10000.0f32);
            }
        }

        index += 1;
    }

    Ok(())
}
