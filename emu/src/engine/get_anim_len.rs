use crate::Fault;

use super::Maanim;

pub fn get_anim_len(anim: &Maanim) -> Result<i32, Fault> {
    if anim.track_count <= 0 {
        return Ok(0);
    }

    let mut length = 0i32;
    let mut track = 0i64;

    loop {
        let entry = anim.tracks.get(track as usize).ok_or(Fault::IndexOutOfRange {
            site: "get_anim_len",
            index: track,
            limit: anim.tracks.len() as i64,
        })?;
        let repeats = entry.header[2];

        if repeats == -1 {
            return Ok(-1);
        }

        let keyframe_count = entry.keyframe_count as i64;

        if keyframe_count != 0 {
            let first = entry.keyframes.first().ok_or(Fault::IndexOutOfRange { site: "get_anim_len", index: 0, limit: 0 })?[0];
            let last = entry.keyframes.get((keyframe_count - 1) as usize).ok_or(Fault::IndexOutOfRange {
                site: "get_anim_len",
                index: keyframe_count - 1,
                limit: entry.keyframes.len() as i64,
            })?[0];
            let end = repeats.wrapping_sub(1).wrapping_mul(last.wrapping_sub(first)).wrapping_add(last);

            if end >= length {
                length = end.wrapping_add(1);
            }
        }

        track += 1;

        if track == anim.track_count as u32 as i64 {
            return Ok(length);
        }
    }
}
