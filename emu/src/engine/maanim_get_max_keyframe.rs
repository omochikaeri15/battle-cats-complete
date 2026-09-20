use crate::Fault;

use super::Maanim;

pub fn maanim_get_max_keyframe(anim: &Maanim) -> Result<i32, Fault> {
    let mut latest = 0i32;
    let mut track = 0i64;

    while track < anim.track_count as i64 {
        let entry = anim
            .tracks
            .get(track as usize)
            .ok_or(Fault::index_out_of_range(track, anim.tracks.len() as i64))?;
        let keyframe_count = entry.keyframe_count as i64;

        if keyframe_count != 0 {
            let last = entry.keyframes.get((keyframe_count - 1) as usize).ok_or(
                Fault::index_out_of_range(keyframe_count - 1, entry.keyframes.len() as i64),
            )?[0];

            if last > latest {
                latest = last;
            }
        }

        track += 1;
    }

    Ok(latest)
}
