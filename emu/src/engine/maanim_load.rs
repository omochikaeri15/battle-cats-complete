use crate::Fault;

use super::{open_asset_stream, read_asset_stream_line, read_csv_cell, read_csv_row, AppContext, AssetStream, Cell};

pub const TRACK_HEADER_CELLS: usize = 3;
pub const KEYFRAME_CELLS: usize = 4;

#[derive(Default, Clone)]
pub struct MaanimTrack {
    pub header: [i32; TRACK_HEADER_CELLS],
    pub keyframe_count: i32,
    pub keyframes: Vec<[i32; KEYFRAME_CELLS]>,
}

#[derive(Clone, Default)]
pub struct Maanim {
    pub tracks: Vec<MaanimTrack>,
    pub track_count: i32,
    pub path: Vec<u8>,
}

pub fn maanim_load(ctx: &mut AppContext, anim: &mut Maanim, path: &[u8]) -> Result<bool, Fault> {
    anim.tracks.clear();
    anim.track_count = 0;
    anim.path.clear();
    anim.path.extend_from_slice(path);

    let Some(bytes) = open_asset_stream(ctx, path, 1, 0)? else {
        return Ok(false);
    };

    let mut stream = AssetStream::new(&bytes, b'\n');
    let stm = &mut stream;

    let mut discarded = Cell { at: 0, len: 0 };
    read_asset_stream_line(stm, &mut discarded);

    read_csv_row(stm);
    let version = read_csv_cell(stm, 0) as i32;

    read_csv_row(stm);
    let track_count = read_csv_cell(stm, 0) as i32;
    anim.track_count = track_count;
    anim.tracks.resize(track_count as i64 as usize, MaanimTrack::default());

    let mut track = 0;

    while track < anim.track_count as i64 {
        read_csv_row(stm);

        anim.tracks[track as usize].header[0] = read_csv_cell(stm, 0) as i32;
        anim.tracks[track as usize].header[1] = read_csv_cell(stm, 1) as i32;
        anim.tracks[track as usize].header[2] = read_csv_cell(stm, 2) as i32;

        read_csv_row(stm);

        let keyframe_count = read_csv_cell(stm, 0) as i32;
        anim.tracks[track as usize].keyframe_count = keyframe_count;
        anim.tracks[track as usize]
            .keyframes
            .resize(keyframe_count as i64 as usize, [0; KEYFRAME_CELLS]);

        let mut keyframe = 0;

        while keyframe < anim.tracks[track as usize].keyframe_count as i64 {
            read_csv_row(stm);

            anim.tracks[track as usize].keyframes[keyframe as usize][0] = read_csv_cell(stm, 0) as i32;
            anim.tracks[track as usize].keyframes[keyframe as usize][1] = read_csv_cell(stm, 1) as i32;
            anim.tracks[track as usize].keyframes[keyframe as usize][2] = read_csv_cell(stm, 2) as i32;

            let mut eased = 0;

            if version > 0 {
                eased = read_csv_cell(stm, 3) as i32;
            }

            anim.tracks[track as usize].keyframes[keyframe as usize][3] = eased;
            keyframe += 1;
        }

        track += 1;
    }

    Ok(true)
}
