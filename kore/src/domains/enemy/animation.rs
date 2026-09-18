use crate::systems::animation::{self, named_offsets, Motion, MotionSet, Loop, RigFiles};
use crate::Vfs;

use super::files;
use super::scanner::EnemyEntry;

const ZOMBIE_MARK: &str = "_zombie";

pub fn set_id(enemy: &EnemyEntry) -> String {
    enemy.id_str()
}

pub fn rig_files(enemy: &EnemyEntry, vfs: &Vfs) -> Option<RigFiles> {
    let plain = files::anim_base_filename(enemy.id);
    let bases = vec![plain.clone(), format!("i{}", plain)];

    animation::rig_files(vfs, &set_id(enemy), &bases)
}

pub fn motions(enemy: &EnemyEntry, vfs: &Vfs) -> MotionSet {
    let plain = files::anim_base_filename(enemy.id);
    let bases = vec![plain.clone(), format!("i{}", plain)];

    let Some(rig) = animation::rigging(vfs, &plain, &bases) else {
        return MotionSet::default();
    };

    let mut motions = Vec::new();

    for (suffix, path) in animation::maanims(vfs, &bases) {
        let zombie = suffix.strip_prefix(ZOMBIE_MARK);
        let index = zombie.unwrap_or(&suffix).parse::<usize>().ok();

        let named = if zombie.is_some() { None } else { index.and_then(animation::standard) };

        motions.push(Motion {
            name: named
                .map(|(name, _, _)| name.to_string())
                .or_else(|| zombie.and(index).and_then(zombie_name).map(str::to_string)),
            slot: named.map(|(_, slot, _)| slot),
            role: named.map(|(_, _, role)| role),
            looping: if named.is_some_and(|(_, _, role)| role.loops()) { Loop::Exact } else { Loop::Frames },
            rig: rig.clone(),
            file: Some(path),
        });
    }

    motions.push(Motion::model(rig));

    MotionSet { name: set_id(enemy), motions, offsets: named_offsets("Base HP") }
}

fn zombie_name(index: usize) -> Option<&'static str> {
    match index {
        0 => Some("Burrow"),
        1 => Some("Dig"),
        2 => Some("Surface"),
        _ => None,
    }
}
