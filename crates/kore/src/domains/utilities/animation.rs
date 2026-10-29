use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domains::settings::FrameCount;
use crate::systems::animation::{Motion, MotionSet, Rigging};
use crate::Source;

pub fn key(png: &Path, cut: &Path, model: &Path, anims: &[PathBuf], frames: FrameCount) -> String {
    let mut key = format!("{:?}|{}", frames, rig_id(png, cut, model));

    for anim in anims {
        key.push('|');
        key.push_str(&anim.to_string_lossy());
    }

    key
}

pub fn motions(png: &Path, cut: &Path, model: &Path, anims: &[PathBuf], frames: FrameCount) -> MotionSet {
    let rig = Arc::new(Rigging {
        id: rig_id(png, cut, model),
        png: Source::disk(png),
        cut: Source::disk(cut),
        model: Source::disk(model),
    });

    let mut motions: Vec<Motion> = anims
        .iter()
        .map(|anim| Motion {
            name: None,
            slot: None,
            role: None,
            looping: frames.looping(),
            rig: rig.clone(),
            file: Some(Source::disk(anim.clone())),
        })
        .collect();

    motions.push(Motion::model(rig));

    MotionSet { name: stem_of(model), motions, offsets: Vec::new() }
}

fn rig_id(png: &Path, cut: &Path, model: &Path) -> String {
    format!("{}|{}|{}", png.display(), cut.display(), model.display())
}

fn stem_of(path: &Path) -> String {
    path.file_stem().map_or_else(|| "animation".to_string(), |stem| stem.to_string_lossy().to_string())
}
