use std::fmt;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use kokoro::domains::sandbox::{Config, Roster};

use crate::platform::Platform;

const STORAGE_KEY: &str = "sandbox";
pub(crate) const QUALITY_RANGE: std::ops::RangeInclusive<u8> = 15..=100;
pub(crate) const QUALITY_STEP: u8 = 5;
const QUALITY_PRESETS: [u8; 6] = [15, 25, 35, 50, 70, 100];
const DEFAULT_QUALITY: u8 = 100;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Volume {
    Off,
    Low,
    #[default]
    Medium,
    High,
}

impl Volume {
    pub(crate) const ALL: [Self; 4] = [Self::Off, Self::Low, Self::Medium, Self::High];

    pub(crate) fn percent(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::Low => 20,
            Self::Medium => 50,
            Self::High => 100,
        }
    }

    pub(crate) fn nearest(percent: i32) -> Self {
        Self::ALL.into_iter().min_by_key(|step| (step.percent() - percent).abs()).unwrap_or_default()
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Device {
    #[default]
    Tablet,
    Phone,
}

impl Device {
    pub(crate) const ALL: [Self; 2] = [Self::Tablet, Self::Phone];
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Tablet => "Tablet",
            Self::Phone => "Phone",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Scaling {
    #[default]
    Stretch,
    Pad,
}

impl Scaling {
    pub(crate) const ALL: [Self; 2] = [Self::Stretch, Self::Pad];
}

impl fmt::Display for Scaling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Stretch => "Stretch",
            Self::Pad => "Pad",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Tab {
    #[default]
    Lineup,
    Stage,
    Config,
    Replay,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Panel {
    #[default]
    Settings,
    Treasure,
    Tech,
    Base,
    Items,
    Invasions,
    Tutorial,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub(crate) struct SandboxState {
    pub(crate) agreement: String,
    pub(crate) tab: Tab,
    pub(crate) panel: Panel,
    pub(crate) roster: Roster,
    pub(crate) config: Config,
    pub(crate) search_query: String,
    pub(crate) music_volume: i32,
    pub(crate) effects_volume: i32,
    pub(crate) two_rows: bool,
    pub(crate) vibrate: bool,
    pub(crate) device: Device,
    pub(crate) image_quality: Option<u8>,
    pub(crate) scaling: Option<Scaling>,
}

impl Default for SandboxState {
    fn default() -> Self {
        Self {
            agreement: String::new(),
            tab: Tab::default(),
            panel: Panel::default(),
            roster: Roster::default(),
            config: Config::default(),
            search_query: String::new(),
            music_volume: Volume::Medium.percent(),
            effects_volume: Volume::Medium.percent(),
            two_rows: true,
            vibrate: false,
            device: Device::default(),
            image_quality: None,
            scaling: None,
        }
    }
}

impl SandboxState {
    pub(crate) fn quality(&self) -> u8 {
        self.image_quality.unwrap_or(DEFAULT_QUALITY).clamp(*QUALITY_RANGE.start(), *QUALITY_RANGE.end())
    }

    pub(crate) fn scaling(&self) -> Scaling {
        self.scaling.unwrap_or_default()
    }

    pub(crate) fn render_scale(&self) -> f32 {
        f32::from(self.quality()) / f32::from(*QUALITY_RANGE.end())
    }

    pub(crate) fn native(&self) -> bool {
        self.quality() >= *QUALITY_RANGE.end()
    }

    pub(crate) fn padded(&self) -> bool {
        !self.native() && self.scaling() == Scaling::Pad
    }

    pub(crate) fn cycle_quality(&mut self) {
        let current = self.quality();

        self.image_quality = Some(QUALITY_PRESETS.into_iter().find(|preset| *preset > current).unwrap_or(*QUALITY_RANGE.start()));
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Usable(pub(crate) Signal<[bool; 6]>);

pub(crate) fn provide(platform: &Platform) -> Signal<SandboxState> {
    let restored = platform.host().recall(STORAGE_KEY).and_then(|json| serde_json::from_str::<SandboxState>(&json).ok()).unwrap_or_default();

    use_context_provider(|| Usable(Signal::new([true; 6])));
    use_context_provider(|| Signal::new(restored))
}

pub(crate) fn usable() -> Usable {
    use_context::<Usable>()
}

pub(crate) fn state() -> Signal<SandboxState> {
    use_context::<Signal<SandboxState>>()
}

pub(crate) fn remember(platform: &Platform, state: &SandboxState) {
    if let Ok(json) = serde_json::to_string(state) {
        platform.host().remember(STORAGE_KEY, &json);
    }
}

pub(crate) fn apply(platform: &Platform, mut state: Signal<SandboxState>, change: impl FnOnce(&mut SandboxState)) {
    change(&mut state.write());
    remember(platform, &state.read());
}
