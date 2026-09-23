use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bind {
    Slot(u8),
    SpeedUp,
    CatCpu,
    Sniper,
    Worker,
    Cannon,
    ZoomOut,
    ZoomIn,
    Left,
    Right,
    Pause,
    Restart,
    Diagnostics,
}

const SLOT_KEYS: [&str; 10] = ["q", "w", "e", "r", "t", "a", "s", "d", "f", "g"];

impl Bind {
    pub const ALL: [Self; 22] = [
        Self::Slot(0),
        Self::Slot(1),
        Self::Slot(2),
        Self::Slot(3),
        Self::Slot(4),
        Self::Slot(5),
        Self::Slot(6),
        Self::Slot(7),
        Self::Slot(8),
        Self::Slot(9),
        Self::SpeedUp,
        Self::CatCpu,
        Self::Sniper,
        Self::Worker,
        Self::Cannon,
        Self::ZoomOut,
        Self::ZoomIn,
        Self::Left,
        Self::Right,
        Self::Pause,
        Self::Restart,
        Self::Diagnostics,
    ];

    pub fn id(self) -> String {
        match self {
            Self::Slot(slot) => format!("slot_{}", slot + 1),
            Self::SpeedUp => "speed_up".to_owned(),
            Self::CatCpu => "cat_cpu".to_owned(),
            Self::Sniper => "sniper".to_owned(),
            Self::Worker => "worker".to_owned(),
            Self::Cannon => "cannon".to_owned(),
            Self::ZoomOut => "zoom_out".to_owned(),
            Self::ZoomIn => "zoom_in".to_owned(),
            Self::Left => "left".to_owned(),
            Self::Right => "right".to_owned(),
            Self::Pause => "pause".to_owned(),
            Self::Restart => "restart".to_owned(),
            Self::Diagnostics => "diagnostics".to_owned(),
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Slot(slot) => format!("Slot {}", slot + 1),
            Self::SpeedUp => "Toggle Speed Up".to_owned(),
            Self::CatCpu => "Toggle Cat CPU".to_owned(),
            Self::Sniper => "Toggle Sniper the Cat".to_owned(),
            Self::Worker => "Level Up Worker Cat".to_owned(),
            Self::Cannon => "Fire Cat Cannon".to_owned(),
            Self::ZoomOut => "Zoom Out".to_owned(),
            Self::ZoomIn => "Zoom In".to_owned(),
            Self::Left => "Move Camera Left".to_owned(),
            Self::Right => "Move Camera Right".to_owned(),
            Self::Pause => "Pause (hold to Escape)".to_owned(),
            Self::Restart => "Restart Battle (double tap)".to_owned(),
            Self::Diagnostics => "Toggle Live Diagnostics".to_owned(),
        }
    }

    pub fn default_key(self) -> &'static str {
        match self {
            Self::Slot(slot) => SLOT_KEYS.get(usize::from(slot)).copied().unwrap_or(""),
            Self::SpeedUp => "i",
            Self::CatCpu => "o",
            Self::Sniper => "p",
            Self::Worker => "Alt",
            Self::Cannon => "Space",
            Self::ZoomOut => "ArrowUp",
            Self::ZoomIn => "ArrowDown",
            Self::Left => "ArrowLeft",
            Self::Right => "ArrowRight",
            Self::Pause => "Escape",
            Self::Restart => "`",
            Self::Diagnostics => "F3",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Keybinds {
    overrides: BTreeMap<String, String>,
}

impl Keybinds {
    pub fn key(&self, bind: Bind) -> &str {
        self.overrides.get(&bind.id()).map_or_else(|| bind.default_key(), String::as_str)
    }

    pub fn bound(&self, key: &str) -> Option<Bind> {
        Bind::ALL.into_iter().find(|bind| !key.is_empty() && self.key(*bind) == key)
    }

    pub fn set(&mut self, bind: Bind, key: &str) {
        if let Some(taken) = self.bound(key).filter(|taken| *taken != bind) {
            self.overrides.insert(taken.id(), String::new());
        }

        self.overrides.insert(bind.id(), key.to_owned());
    }

    pub fn reset(&mut self) {
        self.overrides.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Two actions on one key would make a press ambiguous, so the older holder gives the key up.
    #[test]
    fn rebinding_a_taken_key_unbinds_its_old_action() {
        let mut keys = Keybinds::default();

        assert_eq!(keys.bound("q"), Some(Bind::Slot(0)));

        keys.set(Bind::Cannon, "q");

        assert_eq!(keys.bound("q"), Some(Bind::Cannon));
        assert_eq!(keys.key(Bind::Slot(0)), "");

        keys.reset();

        assert_eq!(keys.key(Bind::Cannon), "Space");
    }

    #[test]
    fn no_two_defaults_share_a_key() {
        let keys = Keybinds::default();

        for bind in Bind::ALL {
            assert_eq!(keys.bound(bind.default_key()), Some(bind));
        }
    }
}
