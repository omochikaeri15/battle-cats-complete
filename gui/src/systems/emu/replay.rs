use emu::runtime::{BattleOptions, CatGod, PartLevels, Seeds, Setup, SetupUnit, StageEntry, TechLevel};
use kore::domains::sandbox::replay::{self as tape, Key};

use super::keys::Action;

pub fn key_of(action: Action) -> Key {
    match action {
        Action::Slot(slot) => Key::Slot(slot.clamp(0, i32::from(u8::MAX)) as u8),
        Action::Item(item) => Key::Item(item.clamp(0, i32::from(u8::MAX)) as u8),
        Action::Worker => Key::Worker,
        Action::Cannon => Key::Cannon,
        Action::ZoomIn => Key::ZoomIn,
        Action::ZoomOut => Key::ZoomOut,
        Action::PanLeft => Key::PanLeft,
        Action::PanRight => Key::PanRight,
        Action::Pause => Key::Pause,
    }
}

pub fn action_of(key: Key) -> Action {
    match key {
        Key::Slot(slot) => Action::Slot(i32::from(slot)),
        Key::Item(item) => Action::Item(i32::from(item)),
        Key::Worker => Action::Worker,
        Key::Cannon => Action::Cannon,
        Key::ZoomIn => Action::ZoomIn,
        Key::ZoomOut => Action::ZoomOut,
        Key::PanLeft => Action::PanLeft,
        Key::PanRight => Action::PanRight,
        Key::Pause => Action::Pause,
    }
}

pub fn seeds_to(seeds: Seeds) -> tape::Seeds {
    tape::Seeds { rng: seeds.rng, entropy: seeds.entropy }
}

pub fn seeds_from(seeds: tape::Seeds) -> Seeds {
    Seeds { rng: seeds.rng, entropy: seeds.entropy }
}

pub fn options_to(options: BattleOptions) -> tape::Options {
    tape::Options {
        music: options.music,
        effects: options.effects,
        two_rows: options.two_rows,
        vibrate: options.vibrate,
    }
}

pub fn options_from(options: tape::Options) -> BattleOptions {
    BattleOptions {
        music: options.music,
        effects: options.effects,
        two_rows: options.two_rows,
        vibrate: options.vibrate,
    }
}

pub fn setup_to(setup: &Setup, map_name: &str, stage_name: &str) -> tape::Setup {
    tape::Setup {
        stage: tape::Stage {
            map_id: setup.stage.map_id,
            stage: setup.stage.stage,
            layout: setup.stage.layout,
            crown: setup.stage.crown,
            map_name: map_name.to_owned(),
            stage_name: stage_name.to_owned(),
        },
        lineup: setup
            .lineup
            .iter()
            .map(|unit| tape::Unit {
                unit: unit.unit,
                form: unit.form,
                level: unit.level,
                plus: unit.plus,
                talents: unit.talents.clone(),
                orbs: unit.orbs.clone(),
            })
            .collect(),
        tech: setup.tech.iter().map(|tech| tape::Level { level: tech.level, plus: tech.plus }).collect(),
        treasures: setup.treasures.iter().map(|chapter| chapter.to_vec()).collect(),
        cannon: setup.cannon,
        style: setup.style,
        foundation: setup.foundation,
        parts: setup
            .parts
            .iter()
            .map(|(&part, levels)| {
                (part, tape::Parts { cannon: levels.cannon, foundation: levels.foundation, style: levels.style })
            })
            .collect(),
        items: setup.items.to_vec(),
        speed_engaged: setup.speed_engaged,
        altar: setup.altar,
        cat_god: match setup.cat_god {
            CatGod::Absent => tape::God::Absent,
            CatGod::Present => tape::God::Present,
            CatGod::Discounted => tape::God::Discounted,
        },
    }
}

pub fn setup_from(saved: &tape::Setup) -> Setup {
    let mut setup = Setup {
        stage: StageEntry {
            map_id: saved.stage.map_id,
            stage: saved.stage.stage,
            layout: saved.stage.layout,
            crown: saved.stage.crown,
        },
        lineup: saved
            .lineup
            .iter()
            .map(|unit| SetupUnit {
                unit: unit.unit,
                form: unit.form,
                level: unit.level,
                plus: unit.plus,
                talents: unit.talents.clone(),
                orbs: unit.orbs.clone(),
            })
            .collect(),
        cannon: saved.cannon,
        style: saved.style,
        foundation: saved.foundation,
        parts: saved
            .parts
            .iter()
            .map(|(&part, levels)| {
                (part, PartLevels { cannon: levels.cannon, foundation: levels.foundation, style: levels.style })
            })
            .collect(),
        speed_engaged: saved.speed_engaged,
        altar: saved.altar,
        cat_god: match saved.cat_god {
            tape::God::Absent => CatGod::Absent,
            tape::God::Present => CatGod::Present,
            tape::God::Discounted => CatGod::Discounted,
        },
        ..Setup::default()
    };

    for (held, tech) in setup.tech.iter_mut().zip(&saved.tech) {
        *held = TechLevel { level: tech.level, plus: tech.plus };
    }

    for (held, chapter) in setup.treasures.iter_mut().zip(&saved.treasures) {
        for (level, saved) in held.iter_mut().zip(chapter) {
            *level = *saved;
        }
    }

    for (held, item) in setup.items.iter_mut().zip(&saved.items) {
        *held = *item;
    }

    setup
}
