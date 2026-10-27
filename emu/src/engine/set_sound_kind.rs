use crate::Fault;

use super::SoundManager;

pub fn set_sound_kind(mgr: &mut SoundManager, sound_id: i32, kind: i32) -> Result<(), Fault> {
    let index = sound_id as i64 as usize;
    let limit = mgr.kinds.len() as i64;

    *mgr.kinds.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, limit))? = kind;

    Ok(())
}
