use std::rc::Rc;

use crate::Fault;

use super::SoundManager;

pub fn stop_audio(mgr: &mut SoundManager, sound_id: i32) -> Result<(), Fault> {
    let every_track = (!sound_id & -3) == 0;
    let mut kind = None;

    if !every_track && sound_id >= 0 {
        let index = sound_id as u32 as usize;
        let limit = mgr.kinds.len() as i64;
        let held = *mgr.kinds.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;

        if held & 1 == 0 {
            kind = Some(held);
        }
    }

    if kind.is_none() {
        if every_track || sound_id >= 0 {
            if let Some(bgm) = mgr.bgm_source.clone() {
                let end = mgr.sources.len();
                let mut slot = 0;

                while slot != end {
                    if mgr.sources[slot].as_ref().is_some_and(|source| Rc::ptr_eq(source, &bgm)) {
                        bgm.stop();
                        mgr.sources[slot] = None;
                        mgr.bgm_source = None;

                        break;
                    }

                    slot += 1;
                }
            }

            let count = mgr.count as i64;
            let mut index = 0i64;

            while index < count {
                let limit = mgr.kinds.len() as i64;

                if *mgr.kinds.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))? & 1 != 0 {
                    let limit = mgr.looping.len() as i64;

                    *mgr.looping.get_mut(index as usize).ok_or(Fault::index_out_of_range(index, limit))? = false;
                }

                index += 1;
            }
        }

        if sound_id.wrapping_add(3) as u32 <= 1 {
            let mut index = 0i64;

            while index < mgr.count as i64 {
                let limit = mgr.kinds.len() as i64;

                if *mgr.kinds.get(index as usize).ok_or(Fault::index_out_of_range(index, limit))? & 2 != 0 {
                    let limit = mgr.sources.len() as i64;
                    let slot = mgr.sources.get_mut(index as usize).ok_or(Fault::index_out_of_range(index, limit))?;

                    if let Some(source) = slot.take() {
                        source.stop();

                        let limit = mgr.looping.len() as i64;

                        *mgr.looping.get_mut(index as usize).ok_or(Fault::index_out_of_range(index, limit))? = false;
                    }
                }

                index += 1;
            }

            return Ok(());
        }

        if sound_id < 0 {
            return Ok(());
        }

        let index = sound_id as u32 as usize;
        let limit = mgr.kinds.len() as i64;

        kind = Some(*mgr.kinds.get(index).ok_or(Fault::index_out_of_range(index as i64, limit))?);
    }

    let index = sound_id as u32 as usize;

    if kind.is_some_and(|kind| kind & 2 != 0) {
        let limit = mgr.sources.len() as i64;
        let slot = mgr.sources.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))?;

        if let Some(source) = slot.take() {
            source.stop();

            let limit = mgr.looping.len() as i64;

            *mgr.looping.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))? = false;
        }
    }

    Ok(())
}
