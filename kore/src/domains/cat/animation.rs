use crate::systems::animation::{self, named_offsets, Motion, MotionSet, Loop, RigFiles};
use crate::Vfs;

use super::files;
use super::scanner::CatEntry;

const SPIRIT_ATTACK: usize = 2;

pub fn set_id(cat: &CatEntry, form: usize) -> String {
    let form_char = match form {
        0 => 'f',
        1 => 'c',
        2 => 's',
        _ => 'u',
    };

    format!("{:03}_{}", cat.id, form_char)
}

pub fn rig_files(cat: &CatEntry, form: usize, vfs: &Vfs) -> Option<RigFiles> {
    let egg_ids = cat.egg_ids.unwrap_or((-1, -1));
    let base = vec![files::anim_base_filename(cat.id, form, egg_ids)];

    animation::rig_files(vfs, &set_id(cat, form), &base)
}

pub fn motions(cat: &CatEntry, form: usize, vfs: &Vfs) -> MotionSet {
    let egg_ids = cat.egg_ids.unwrap_or((-1, -1));
    let base = vec![files::anim_base_filename(cat.id, form, egg_ids)];
    let id = set_id(cat, form);

    let mut motions = Vec::new();

    if let Some(rig) = animation::rigging(vfs, &id, &base) {
        for (suffix, path) in animation::maanims(vfs, &base) {
            let index = suffix.parse::<usize>().ok();
            let named = index.and_then(animation::standard);

            motions.push(Motion {
                name: named.map(|(name, _, _)| name.to_string()),
                slot: named.map(|(_, slot, _)| slot),
                role: named.map(|(_, _, role)| role),
                looping: if named.is_some_and(|(_, _, role)| role.loops()) { Loop::Exact } else { Loop::Frames },
                rig: rig.clone(),
                file: Some(path),
            });
        }

        motions.push(Motion::model(rig));
    }

    if let Some(spirit) = spirit_motion(cat, form, vfs) {
        motions.push(spirit);
    }

    MotionSet { name: id, motions, offsets: named_offsets("Gacha") }
}

fn spirit_motion(cat: &CatEntry, form: usize, vfs: &Vfs) -> Option<Motion> {
    let conjure_id = cat.stats.get(form)?.as_ref()?.conjure_unit_id;

    if conjure_id <= 0 {
        return None;
    }

    let spirit_id = conjure_id as u32;
    let base = vec![files::anim_base_filename(spirit_id, 0, (-1, -1))];
    let rig = animation::rigging(vfs, &format!("spirit_{}", spirit_id), &base)?;
    let anim = vfs.find(&files::maanim_file(spirit_id, 0, (-1, -1), SPIRIT_ATTACK)).map(|path| vfs.source(&path))?;

    Some(Motion {
        name: Some("Spirit".to_string()),
        slot: None,
        role: None,
        looping: Loop::Frames,
        rig,
        file: Some(anim),
    })
}
