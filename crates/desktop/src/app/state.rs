use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use codex::chapter::Category;

use kokoro::systems::animation::export::ExportFormat;
use kokoro::systems::animation::Placement;
use kokoro::domains::import::ImportSubTab;
use kokoro::domains::stage::{GlobalMapId, GlobalStageId};

use crate::domains::files::Mode;
use crate::widget::popup::StoredSize;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppState {
    pub(crate) cat: CatListState,
    pub(crate) enemy: EnemyListState,
    pub(crate) stage: StageListState,
    pub(crate) mods: ModsListState,
    pub(crate) files: FilesState,
    pub(crate) data: GameDataState,
    pub(crate) animation: AnimState,
    pub(crate) background: BackgroundState,
    pub(crate) studio: StudioState,
    pub(crate) notice: NoticeState,
    pub(crate) help: HelpState,
    pub(crate) sandbox: SandboxState,
    pub(crate) source: SourceState,
    pub(crate) popups: BTreeMap<String, StoredSize>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct StudioState {
    pub name: String,
    pub sheet: Option<PathBuf>,
    pub cuts: Option<PathBuf>,
    pub model: Option<PathBuf>,
    pub anims: Vec<PathBuf>,
    pub sealed: bool,
    pub target_mod: Option<String>,
    pub atlas: bool,
    #[serde(alias = "clip")]
    pub motion: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct SourceState {
    pub agreement: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct NoticeState {
    pub acknowledged: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SandboxVolume {
    Off,
    Low,
    #[default]
    Medium,
    High,
}

impl SandboxVolume {
    pub const ALL: [Self; 4] = [Self::Off, Self::Low, Self::Medium, Self::High];

    pub fn percent(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::Low => 20,
            Self::Medium => 50,
            Self::High => 100,
        }
    }

    pub fn nearest(percent: i32) -> Self {
        Self::ALL
            .into_iter()
            .min_by_key(|step| (step.percent() - percent).abs())
            .unwrap_or_default()
    }
}

impl std::fmt::Display for SandboxVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        })
    }
}

pub(crate) const QUALITY_RANGE: std::ops::RangeInclusive<u8> = 15..=100;
pub(crate) const QUALITY_STEP: u8 = 5;
const QUALITY_PRESETS: [u8; 6] = [15, 25, 35, 50, 70, 100];
const CPU_QUALITY: u8 = 35;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SandboxDevice {
    #[default]
    Tablet,
    Phone,
}

impl SandboxDevice {
    pub const ALL: [Self; 2] = [Self::Tablet, Self::Phone];
}

impl std::fmt::Display for SandboxDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Tablet => "Tablet",
            Self::Phone => "Phone",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SandboxScaling {
    #[default]
    Stretch,
    Pad,
}

impl SandboxScaling {
    pub const ALL: [Self; 2] = [Self::Stretch, Self::Pad];
}

impl std::fmt::Display for SandboxScaling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Stretch => "Stretch",
            Self::Pad => "Pad",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SandboxTab {
    #[default]
    Lineup,
    Stage,
    Config,
    Replay,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SandboxPanel {
    #[default]
    Settings,
    Treasure,
    Tech,
    Base,
    Items,
    Invasions,
    Tutorial,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct SandboxState {
    pub agreement: String,
    pub tab: SandboxTab,
    pub panel: SandboxPanel,
    pub roster: kokoro::domains::sandbox::Roster,
    pub config: kokoro::domains::sandbox::Config,
    pub list_scroll_offset: f32,
    pub search_query: String,
    pub music_volume: i32,
    pub effects_volume: i32,
    pub two_rows: bool,
    pub vibrate: bool,
    pub device: SandboxDevice,
    pub usable_items: [bool; 6],
    pub image_quality: Option<u8>,
    pub scaling: Option<SandboxScaling>,
}

impl Default for SandboxState {
    fn default() -> Self {
        Self {
            agreement: String::new(),
            tab: SandboxTab::default(),
            panel: SandboxPanel::default(),
            roster: kokoro::domains::sandbox::Roster::default(),
            config: kokoro::domains::sandbox::Config::default(),
            list_scroll_offset: 0.0,
            search_query: String::new(),
            music_volume: SandboxVolume::Medium.percent(),
            effects_volume: SandboxVolume::Medium.percent(),
            two_rows: false,
            vibrate: false,
            device: SandboxDevice::default(),
            usable_items: [true; 6],
            image_quality: None,
            scaling: None,
        }
    }
}

impl SandboxState {
    pub fn quality(&self) -> u8 {
        self.image_quality.unwrap_or(*QUALITY_RANGE.end()).clamp(*QUALITY_RANGE.start(), *QUALITY_RANGE.end())
    }

    pub fn scaling(&self) -> SandboxScaling {
        self.scaling.unwrap_or_default()
    }

    pub fn settle_display(&mut self, cpu: bool) {
        let (quality, scaling) = if cpu { (CPU_QUALITY, SandboxScaling::Pad) } else { (*QUALITY_RANGE.end(), SandboxScaling::Stretch) };

        self.image_quality.get_or_insert(quality);
        self.scaling.get_or_insert(scaling);
    }

    pub fn render_scale(&self) -> f32 {
        f32::from(self.quality()) / f32::from(*QUALITY_RANGE.end())
    }

    pub fn native(&self) -> bool {
        self.quality() >= *QUALITY_RANGE.end()
    }

    pub fn padded(&self) -> bool {
        !self.native() && self.scaling() == SandboxScaling::Pad
    }

    pub fn cycle_quality(&mut self) {
        let current = self.quality();

        self.image_quality = Some(QUALITY_PRESETS.into_iter().find(|preset| *preset > current).unwrap_or(*QUALITY_RANGE.start()));
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct HelpState {
    pub active_page: usize,
    pub scroll_offset: f32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct ModsListState {
    pub selected_mod: Option<String>,
    pub search_query: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct FilesState {
    pub mount: Option<String>,
    pub mode: Mode,
    pub search_query: String,
    pub selected_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct CatListState {
    pub list_scroll_offset: f32,
    pub selected_cat: Option<u32>,
    pub selected_form: usize,
    pub search_query: String,
    pub talent_levels: HashMap<u32, HashMap<u8, u8>>,
    pub talent_history: VecDeque<u32>,
    pub current_level: i32,
    pub level_input: String,
}

impl Default for CatListState {
    fn default() -> Self {
        Self {
            list_scroll_offset: 0.0,
            selected_cat: None,
            selected_form: 0,
            search_query: String::new(),
            talent_levels: HashMap::new(),
            talent_history: VecDeque::new(),
            current_level: 1,
            level_input: String::from("1"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct EnemyListState {
    pub list_scroll_offset: f32,
    pub selected_enemy: Option<u32>,
    pub search_query: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct StageListState {
    pub selected_category: Option<Category>,
    pub selected_map: Option<GlobalMapId>,
    pub selected_stage: Option<GlobalStageId>,
    pub selected_crown: u8,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct GameDataState {
    pub adb_import_type_idx: usize,
    pub adb_region_idx: usize,
    pub selected_import: Option<ImportSubTab>,
}

impl Default for GameDataState {
    fn default() -> Self {
        Self {
            adb_import_type_idx: 0,
            adb_region_idx: 4,
            selected_import: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct AnimState {
    pub last_export_format: ExportFormat,
    pub last_export_quality: Option<i32>,
    pub last_export_compression: Option<i32>,
    pub controls_expanded: bool,
    pub export_background: bool,
    pub include_debug: bool,
    pub placement: Placement,
    pub pan: (f32, f32),
    pub zoom: f32,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            last_export_format: ExportFormat::Gif,
            last_export_quality: None,
            last_export_compression: None,
            controls_expanded: true,
            export_background: false,
            include_debug: false,
            placement: Placement::default(),
            pan: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct BackgroundState {
    pub last_export_format: ExportFormat,
    pub last_export_quality: Option<i32>,
    pub last_export_compression: Option<i32>,
    pub controls_expanded: bool,
}

impl Default for BackgroundState {
    fn default() -> Self {
        Self {
            last_export_format: ExportFormat::Gif,
            last_export_quality: None,
            last_export_compression: None,
            controls_expanded: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_file_from_before_the_shared_camera_still_opens_at_a_usable_zoom() {
        // A zoom of zero would deserialize as a blank viewport rather than a default one.
        let held: AnimState = serde_json::from_str(r#"{"controls_expanded":true}"#).expect("it loads");

        assert_eq!(held.zoom, 1.0);
        assert_eq!(held.pan, (0.0, 0.0));
    }

    #[test]
    fn a_state_file_from_before_the_debug_export_leaves_it_switched_off() {
        // It opts a render into drawing overlays, so it has to default to off for
        // everyone who never asked for it, however old their state file is.
        let held: AnimState = serde_json::from_str(r#"{"controls_expanded":true}"#).expect("it loads");

        assert!(!held.include_debug);

        let held: AnimState = serde_json::from_str(r#"{"include_debug":true}"#).expect("it loads");

        assert!(held.include_debug, "and a deliberate choice round-trips");
    }

    #[test]
    fn the_background_preference_survives_a_restart_transparent_by_default() {
        let held: AnimState = serde_json::from_str(r#"{"controls_expanded":true}"#).expect("it loads");

        assert!(!held.export_background, "a format that can hold alpha still exports with it");

        let held: AnimState = serde_json::from_str(r#"{"export_background":true}"#).expect("it loads");

        assert!(held.export_background);
    }

    #[test]
    fn a_saved_offset_row_is_dropped_so_the_placement_default_wins() {
        // Older builds persisted the bare view as `offset_row: null`, and a null that
        // survives means the viewer keeps showing a placement the engine never draws.
        let held: AnimState = serde_json::from_str(r#"{"offset_row":null}"#).expect("it loads");

        assert_eq!(held.placement, Placement::Row(0));

        let held: AnimState = serde_json::from_str(r#"{"placement":"Bare"}"#).expect("it loads");

        assert_eq!(held.placement, Placement::Bare, "a deliberate choice still round-trips");
    }

    #[test]
    fn hardware_defaults_fill_in_once_and_never_override_a_choice() {
        // A fresh install drawn on the CPU starts low and padded.
        let mut fresh = SandboxState::default();
        fresh.settle_display(true);
        assert_eq!((fresh.quality(), fresh.scaling()), (35, SandboxScaling::Pad));

        // Settling again on better hardware later must not undo the first pick.
        fresh.settle_display(false);
        assert_eq!((fresh.quality(), fresh.scaling()), (35, SandboxScaling::Pad));

        // A save from before this existed already holds a choice, so it is left alone.
        let mut saved: SandboxState = serde_json::from_str(r#"{"image_quality":100,"scaling":"Stretch"}"#).expect("it loads");
        saved.settle_display(true);
        assert_eq!((saved.quality(), saved.scaling()), (100, SandboxScaling::Stretch));
    }
}
