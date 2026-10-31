use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use emulator::runtime::{BATTLE_ITEMS, BattleOptions, CatGod, PartLevels, Setup, SetupUnit, StageEntry, StageVariant, TECH_COUNT, TREASURE_CHAPTERS, TREASURE_STAGES, TechLevel, TutorialFlags};

use kokoro::domains::sandbox::replay::Save;
use kokoro::domains::settings::ReplaySource;

use crate::host::{Action, Diagnostics, Frame, Phase, Touch};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariantWire {
    Plain,
    Outbreak,
    Invasion,
    ZombieInvasion,
    ExInvasion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageWire {
    pub map_id: i32,
    pub stage: i32,
    pub layout: Option<i32>,
    pub crown: i32,
    pub dungeon_stage: Option<i32>,
    pub variant: VariantWire,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitWire {
    pub unit: i32,
    pub form: i32,
    pub level: u32,
    pub plus: u32,
    pub talents: Vec<(i32, i32)>,
    pub orbs: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatGodWire {
    Absent,
    Present,
    Discounted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupWire {
    pub stage: StageWire,
    pub lineup: Vec<UnitWire>,
    pub tech: Vec<(u32, u32)>,
    pub treasures: Vec<Vec<i32>>,
    pub cannon: i32,
    pub style: i32,
    pub foundation: i32,
    pub parts: Vec<(i32, i32, i32, i32)>,
    pub items: Vec<bool>,
    pub speed_engaged: bool,
    pub altar: Option<i32>,
    pub cat_god: CatGodWire,
    pub tutorial: [bool; 5],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionsWire {
    pub music: i32,
    pub effects: i32,
    pub two_rows: bool,
    pub vibrate: bool,
}

impl From<BattleOptions> for OptionsWire {
    fn from(options: BattleOptions) -> Self {
        Self { music: options.music, effects: options.effects, two_rows: options.two_rows, vibrate: options.vibrate }
    }
}

impl From<OptionsWire> for BattleOptions {
    fn from(wire: OptionsWire) -> Self {
        Self { music: wire.music, effects: wire.effects, two_rows: wire.two_rows, vibrate: wire.vibrate }
    }
}

impl From<StageEntry> for StageWire {
    fn from(entry: StageEntry) -> Self {
        let variant = match entry.variant {
            StageVariant::Plain => VariantWire::Plain,
            StageVariant::Outbreak => VariantWire::Outbreak,
            StageVariant::Invasion => VariantWire::Invasion,
            StageVariant::ZombieInvasion => VariantWire::ZombieInvasion,
            StageVariant::ExInvasion => VariantWire::ExInvasion,
        };

        Self { map_id: entry.map_id, stage: entry.stage, layout: entry.layout, crown: entry.crown, dungeon_stage: entry.dungeon_stage, variant }
    }
}

impl From<StageWire> for StageEntry {
    fn from(wire: StageWire) -> Self {
        let variant = match wire.variant {
            VariantWire::Plain => StageVariant::Plain,
            VariantWire::Outbreak => StageVariant::Outbreak,
            VariantWire::Invasion => StageVariant::Invasion,
            VariantWire::ZombieInvasion => StageVariant::ZombieInvasion,
            VariantWire::ExInvasion => StageVariant::ExInvasion,
        };

        Self { map_id: wire.map_id, stage: wire.stage, layout: wire.layout, crown: wire.crown, dungeon_stage: wire.dungeon_stage, variant }
    }
}

impl From<&Setup> for SetupWire {
    fn from(setup: &Setup) -> Self {
        Self {
            stage: setup.stage.into(),
            lineup: setup
                .lineup
                .iter()
                .map(|unit| UnitWire { unit: unit.unit, form: unit.form, level: unit.level, plus: unit.plus, talents: unit.talents.clone(), orbs: unit.orbs.clone() })
                .collect(),
            tech: setup.tech.iter().map(|tech| (tech.level, tech.plus)).collect(),
            treasures: setup.treasures.iter().map(|chapter| chapter.to_vec()).collect(),
            cannon: setup.cannon,
            style: setup.style,
            foundation: setup.foundation,
            parts: setup.parts.iter().map(|(part, levels)| (*part, levels.cannon, levels.foundation, levels.style)).collect(),
            items: setup.items.to_vec(),
            speed_engaged: setup.speed_engaged,
            altar: setup.altar,
            cat_god: match setup.cat_god {
                CatGod::Absent => CatGodWire::Absent,
                CatGod::Present => CatGodWire::Present,
                CatGod::Discounted => CatGodWire::Discounted,
            },
            tutorial: [setup.tutorial.battle_cleared, setup.tutorial.deck_seen, setup.tutorial.two_rows_seen, setup.tutorial.cat_god_seen, setup.tutorial.shop_seen],
        }
    }
}

impl From<SetupWire> for Setup {
    fn from(wire: SetupWire) -> Self {
        let mut setup = Setup { stage: wire.stage.into(), ..Setup::default() };

        setup.lineup = wire
            .lineup
            .into_iter()
            .map(|unit| SetupUnit { unit: unit.unit, form: unit.form, level: unit.level, plus: unit.plus, talents: unit.talents, orbs: unit.orbs })
            .collect();

        for (slot, (level, plus)) in wire.tech.into_iter().take(TECH_COUNT).enumerate() {
            setup.tech[slot] = TechLevel { level, plus };
        }

        for (chapter, levels) in wire.treasures.into_iter().take(TREASURE_CHAPTERS).enumerate() {
            for (stage, level) in levels.into_iter().take(TREASURE_STAGES).enumerate() {
                setup.treasures[chapter][stage] = level;
            }
        }

        setup.cannon = wire.cannon;
        setup.style = wire.style;
        setup.foundation = wire.foundation;
        setup.parts = wire.parts.into_iter().map(|(part, cannon, foundation, style)| (part, PartLevels { cannon, foundation, style })).collect::<BTreeMap<_, _>>();

        for (slot, stocked) in wire.items.into_iter().take(BATTLE_ITEMS).enumerate() {
            setup.items[slot] = stocked;
        }

        setup.speed_engaged = wire.speed_engaged;
        setup.altar = wire.altar;
        setup.cat_god = match wire.cat_god {
            CatGodWire::Absent => CatGod::Absent,
            CatGodWire::Present => CatGod::Present,
            CatGodWire::Discounted => CatGod::Discounted,
        };
        setup.tutorial = TutorialFlags {
            battle_cleared: wire.tutorial[0],
            deck_seen: wire.tutorial[1],
            two_rows_seen: wire.tutorial[2],
            cat_god_seen: wire.tutorial[3],
            shop_seen: wire.tutorial[4],
        };

        setup
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReelWire {
    Latest,
    Held,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recorded {
    pub save: Save,
    pub frames: usize,
    pub assets: usize,
    pub bytes: u64,
    pub uploaded: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Launch {
    pub setup: SetupWire,
    pub options: OptionsWire,
    pub phone: bool,
    pub width: f32,
    pub height: f32,
    pub prefixes: Vec<Box<str>>,
    pub pools: Vec<Box<str>>,
    pub priority: Vec<String>,
    pub record: bool,
    pub app: String,
    pub map: String,
    pub stage: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleCommand {
    Prepare { fonts: Vec<Box<str>> },
    Start(Box<Launch>),
    Watch { reel: ReelWire, checked: bool, source: ReplaySource, phone: bool, width: f32, height: f32, priority: Vec<String> },
    Latest,
    ExportLatest,
    Upload { name: String, bytes: Vec<u8> },
    DeleteLatest,
    ExportVideo(Box<Filming>),
    AbortVideo,
    Film { source: ReplaySource, priority: Vec<String> },
    Roll(u32),
    Unroll,
    Stop,
    Forget,
    Resume,
    Restart,
    Touches(Vec<Touch>),
    Key(Action, bool),
    Resize(f32, f32),
    Phone(bool),
    Volumes(i32, i32),
    Presented,
    Diagnose(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioCommand {
    Effect { clip: Box<str>, channels: u16, rate: u32, samples: Vec<f32> },
    Track { clip: Box<str>, bytes: Vec<u8> },
    Start { voice: u32, clip: Box<str>, looped: bool, level: f32 },
    Level { voice: u32, level: f32 },
    Stop { voice: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetWire {
    pub name: Box<str>,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Filming {
    pub source: ReplaySource,
    pub priority: Vec<String>,
    pub quality: u32,
    pub compression: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rolled {
    pub size: (u32, u32),
    pub frames: Vec<Frame>,
    pub sheets: Vec<SheetWire>,
    pub audio: Vec<(u32, AudioCommand)>,
    pub played: usize,
    pub total: usize,
    pub over: bool,
    pub failure: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Filmed {
    Done,
    Aborted,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    pub phase: Phase,
    pub covered: bool,
    pub surfaced: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleEvent {
    Standing(Standing),
    Frame(Frame),
    Sheets(Vec<SheetWire>),
    Audio(Vec<AudioCommand>),
    Options(OptionsWire),
    Diagnostics(Diagnostics),
    Failure(String),
    Warming { done: usize, total: usize },
    Latest(Option<Box<Recorded>>),
    Exporting(f32),
    Exported(Result<Vec<u8>, String>),
    Uploading(f32),
    Uploaded(Result<(), String>),
    Rolled(Box<Rolled>),
    Rendering(f32),
    Sounding(f32),
    Filmed(Filmed),
    Foreign(String),
    Rejected(String),
    Cut,
}

pub type BattleWatch = Box<dyn Fn(BattleEvent)>;
