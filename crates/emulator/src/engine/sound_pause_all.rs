use crate::Fault;

use super::SoundManager;

pub fn sound_pause_all(mgr: &mut SoundManager) -> Result<(), Fault> {
    mgr.paused = 1;

    if let Some(bgm) = &mgr.bgm_source {
        bgm.stop();
    }

    let mut index = 0i64;

    while index < mgr.count as i64 {
        let limit = mgr.kinds.len() as i64;

        if *mgr.kinds.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))? & 2 != 0 {
            let limit = mgr.sources.len() as i64;
            let slot = mgr.sources.get_mut(index as usize).ok_or(Fault::index_out_of_range(index, limit))?;

            if let Some(source) = slot.take() {
                source.stop();
            }
        }

        index += 1;
    }

    Ok(())
}
