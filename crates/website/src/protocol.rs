use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::enemy::scanner::EnemyEntry;
use kokoro::domains::settings::ScannerConfig;
use kokoro::domains::stage::scanner::StageBundle;
use omote::platform::battle::{BattleCommand, BattleEvent};
use omote::platform::mods::{ModCommand, ModState};

pub(crate) const HELLO: u32 = 0;

#[derive(Serialize, Deserialize)]
pub(crate) struct Call {
    pub(crate) id: u32,
    pub(crate) request: Request,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum PictureKind {
    CatBanner,
    EnemyBanner,
    Trimmed,
    Boxed(u32),
    Cut { sheet: Box<str>, icon: u32 },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RigWire {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
    pub(crate) version: i32,
    pub(crate) image_name: String,
    pub(crate) cuts: Vec<(i32, i32, i32, i32, String)>,
    pub(crate) opaque: Vec<Option<(i32, i32, i32, i32)>>,
    pub(crate) hull: Vec<Vec<(i32, i32)>>,
    pub(crate) model: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Prepared {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) png: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum Request {
    Battle(BattleCommand),
    Names,
    Fetch(Vec<Box<str>>),
    Picture { kind: PictureKind, name: Box<str> },
    Warm(Vec<Box<str>>),
    Rig { png: Box<str>, cut: Box<str>, model: Box<str> },
    Cats(ScannerConfig),
    Enemies(ScannerConfig),
    Stages(ScannerConfig),
    Mods(ModCommand),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Answer {
    pub(crate) id: u32,
    pub(crate) reply: Reply,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum Reply {
    Ready,
    Names(Vec<Box<str>>),
    Files(Vec<(Box<str>, Vec<u8>)>),
    Picture(Option<Prepared>),
    Warmed,
    Rig(Option<RigWire>),
    Cats(Vec<CatEntry>),
    Enemies(Vec<EnemyEntry>),
    Stages(Box<StageBundle>),
    Mods(ModState),
    Progress { done: usize, total: usize },
    Phase { label: Box<str>, done: usize, total: usize },
    Battle(BattleEvent),
    Failed(String),
}

pub(crate) fn encode<T: Serialize>(value: &T) -> Option<Vec<u8>> {
    postcard::to_allocvec(value).ok()
}

pub(crate) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    postcard::from_bytes(bytes).ok()
}
