use serde::{Deserialize, Serialize};

pub const CHAPTERS: [&str; 9] = [
    "Empire of Cats 1",
    "Empire of Cats 2",
    "Empire of Cats 3",
    "Into the Future 1",
    "Into the Future 2",
    "Into the Future 3",
    "Cats of the Cosmos 1",
    "Cats of the Cosmos 2",
    "Cats of the Cosmos 3",
];

pub const FULL_PERCENT: u32 = 100;

pub const ITEMS: [&str; 6] = ["Speed Up", "Treasure Radar", "Rich Cat", "Cat CPU", "Cat Jobs", "Sniper the Cat"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tech {
    pub name: &'static str,
    pub slot: usize,
    pub level: u32,
    pub plus: u32,
}

pub const TECHS: [Tech; 10] = [
    Tech { name: "Cat Cannon Attack", slot: 1, level: 20, plus: 10 },
    Tech { name: "Cat Cannon Range", slot: 2, level: 10, plus: 0 },
    Tech { name: "Cat Cannon Charge", slot: 3, level: 20, plus: 10 },
    Tech { name: "Worker Cat Efficiency", slot: 4, level: 20, plus: 10 },
    Tech { name: "Worker Cat Wallet", slot: 5, level: 20, plus: 10 },
    Tech { name: "Cat Base Health", slot: 6, level: 20, plus: 10 },
    Tech { name: "Research", slot: 7, level: 20, plus: 10 },
    Tech { name: "Accounting", slot: 8, level: 20, plus: 10 },
    Tech { name: "Study", slot: 9, level: 20, plus: 10 },
    Tech { name: "Cat Energy", slot: 10, level: 20, plus: 10 },
];

impl Tech {
    pub fn hint(&self) -> String {
        if self.plus > 0 { format!("{}+{}", self.level, self.plus) } else { self.level.to_string() }
    }

    pub fn resolve(&self, entry: &str) -> (u32, u32) {
        let mut terms = entry.split('+').map(|term| term.trim().parse::<u32>().ok());

        let Some(Some(level)) = terms.next() else {
            return (self.level, self.plus);
        };

        let plus = terms.flatten().sum::<u32>();

        (level.clamp(1, self.level), plus.min(self.plus))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatGod {
    Absent,
    #[default]
    Present,
    Discounted,
}

impl CatGod {
    pub const ALL: [Self; 3] = [Self::Absent, Self::Present, Self::Discounted];
}

impl std::fmt::Display for CatGod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Absent => "Absent",
            Self::Present => "Present",
            Self::Discounted => "Discounted",
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub treasures: [String; 9],
    pub techs: [String; 10],
    pub cannon: Option<i32>,
    pub style: Option<i32>,
    pub foundation: Option<i32>,
    pub cannon_level: String,
    pub style_level: String,
    pub foundation_level: String,
    pub items_off: [bool; 6],
    pub altar_level: String,
    pub cat_god: CatGod,
}

impl Config {
    pub fn treasure(&self, chapter: usize) -> u32 {
        self.treasures
            .get(chapter)
            .and_then(|entry| entry.trim().trim_end_matches('%').trim().parse::<u32>().ok())
            .map_or(FULL_PERCENT, |percent| percent.min(FULL_PERCENT))
    }

    pub fn altar(&self) -> Option<i32> {
        self.altar_level.trim().parse::<i32>().ok().map(|level| level.max(1))
    }

    pub fn tech(&self, index: usize) -> (u32, u32) {
        TECHS.get(index).map_or((1, 0), |tech| tech.resolve(self.techs.get(index).map_or("", String::as_str)))
    }
}

pub fn level(entry: &str, highest: i32) -> i32 {
    entry.trim().parse::<i32>().map_or(highest, |level| level.clamp(1, highest.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // An untouched field has to mean the vanilla cap, not zero.
    #[test]
    fn an_empty_entry_is_the_games_maximum() {
        let config = Config::default();

        assert_eq!(config.treasure(0), 100);
        assert_eq!(config.tech(0), (20, 10));
        assert_eq!(config.tech(1), (10, 0), "cannon range stops at ten and has no plus levels");
        assert_eq!(level("", 30), 30);
        assert_eq!(config.altar(), None, "no entry means the altar is already destroyed");
        assert_eq!(config.cat_god, CatGod::Present);
    }

    #[test]
    fn an_entry_past_the_cap_is_pulled_back() {
        let mut config = Config::default();

        config.techs[0] = "99+99".to_owned();
        config.treasures[2] = "250%".to_owned();

        assert_eq!(config.tech(0), (20, 10));
        assert_eq!(config.treasure(2), 100);
        assert_eq!(level("45", 30), 30);
    }
}
