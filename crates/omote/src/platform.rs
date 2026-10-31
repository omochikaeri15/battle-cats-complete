pub mod battle;
pub mod mods;

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use codex::graphics::animate::FrameData;
use codex::graphics::rig::SpriteSheet;
use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::enemy::scanner::EnemyEntry;
use kokoro::domains::settings::ScannerConfig;
use kokoro::domains::stage::scanner::StageBundle;

pub const LOGGING_KEY: &str = "logging";
pub const LOGGING_ON: &str = "on";

pub type Pending<T> = Pin<Box<dyn Future<Output = T>>>;
pub type Bytes = Arc<[u8]>;
pub type Report = Box<dyn Fn(Progress)>;
pub type Watch = Box<dyn Fn(Phase)>;
pub type Fetched = Vec<(Box<str>, Bytes)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Picture {
    CatBanner,
    EnemyBanner,
    Trimmed,
    Boxed(u32),
    Cut { sheet: Box<str>, icon: u32 },
}

pub struct Prepared {
    pub width: u32,
    pub height: u32,
    pub png: Bytes,
}

pub struct Picked {
    pub name: String,
    pub size: u64,
    pub bytes: Option<Bytes>,
}

pub struct RigParts {
    pub sheet: SpriteSheet,
    pub model: Bytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script {
    Japanese,
    Korean,
    TraditionalChinese,
    Thai,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault(pub String);

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase {
    pub label: Box<str>,
    pub done: usize,
    pub total: usize,
}

#[derive(Clone)]
pub struct Painted {
    pub frame: FrameData,
    pub tint: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Line { from: (f32, f32), to: (f32, f32), color: [f32; 4], width: f32 },
    Circle { center: (f32, f32), radius: f32, color: [f32; 4] },
    Rect { min: (f32, f32), max: (f32, f32), fill: [f32; 4], stroke: Option<([f32; 4], f32)> },
}

pub struct Atlas<'a> {
    pub key: usize,
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

pub struct Scene<'a> {
    pub atlas: Option<Atlas<'a>>,
    pub parts: &'a [Painted],
    pub shapes: &'a [Shape],
    pub pan: (f32, f32),
    pub zoom: f32,
    pub clear: [f32; 4],
}

pub struct Captured {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub trait Surface {
    fn resize(&self, width: u32, height: u32, scale: f32);
    fn draw(&self, scene: &Scene<'_>);
    fn capture(&self, scene: &Scene<'_>) -> Pending<Result<Captured, Fault>>;
}

pub trait Host {
    fn recall(&self, key: &str) -> Option<String>;
    fn remember(&self, key: &str, value: &str);
    fn pixel_ratio(&self) -> f32;
    fn frame(&self) -> Pending<f64>;
    fn now(&self) -> f64;
    fn surface(&self, canvas: &str, overlay: &str) -> Pending<Result<Rc<dyn Surface>, Fault>>;
    fn names(&self, watch: Watch) -> Pending<Result<Vec<Box<str>>, Fault>>;
    fn fetch(&self, names: Vec<Box<str>>) -> Pending<Result<Fetched, Fault>>;
    fn picture(&self, kind: Picture, name: Box<str>) -> Pending<Result<Option<Prepared>, Fault>>;
    fn warm(&self, names: Vec<Box<str>>) -> Pending<()>;
    fn rig(&self, png: Box<str>, cut: Box<str>, model: Box<str>) -> Pending<Result<Option<RigParts>, Fault>>;
    fn font(&self, script: Script) -> Pending<Result<Bytes, Fault>>;
    fn download(&self, name: &str, mime: &str, bytes: Bytes);
    fn can_copy(&self) -> bool;
    fn copy_image(&self, png: Bytes) -> Pending<Result<(), Fault>>;
    fn copy_text(&self, text: &str) -> Pending<Result<(), Fault>>;
    fn capture_pointer(&self, element: &str, pointer: i32);
    fn pick_file(&self, accept: &str, limit: u64) -> Pending<Result<Option<Picked>, Fault>>;
    fn can_film(&self) -> bool;
    fn fullscreen(&self) -> bool;
    fn set_fullscreen(&self, on: bool) -> Result<(), Fault>;
    fn fullscreen_watch(&self, watch: Box<dyn Fn(bool)>);
    fn mods(&self, command: mods::ModCommand, report: Report) -> Pending<Result<mods::ModState, Fault>>;
    fn sleep(&self, millis: u32) -> Pending<()>;
    fn can_publish(&self) -> bool;
    fn publish_image(&self, folder: &str, name: &str, png: Bytes) -> Pending<Result<Option<String>, Fault>>;
    fn cats(&self, config: ScannerConfig, report: Report) -> Pending<Result<Vec<CatEntry>, Fault>>;
    fn enemies(&self, config: ScannerConfig, report: Report) -> Pending<Result<Vec<EnemyEntry>, Fault>>;
    fn stages(&self, config: ScannerConfig, report: Report) -> Pending<Result<StageBundle, Fault>>;
    fn battle(&self, command: battle::BattleCommand);
    fn battle_watch(&self, watch: battle::BattleWatch);
    fn battle_canvas(&self, canvas: &str);
    fn battle_display(&self, scale: f32, padded: bool);
}

#[derive(Clone)]
pub struct Platform {
    host: Rc<dyn Host>,
}

impl PartialEq for Platform {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.host, &other.host)
    }
}

impl Platform {
    pub fn new(host: impl Host + 'static) -> Self {
        Self { host: Rc::new(host) }
    }

    pub fn host(&self) -> &dyn Host {
        self.host.as_ref()
    }
}
