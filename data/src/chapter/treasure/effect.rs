//! The effects treasure sets grant, and how the engine applies them.

use serde::{Deserialize, Serialize};

/// The effect a treasure set grants, keyed by the identifier in its effect column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreasureEffect {
    /// Raises the Cat Base's health.
    BaseHealth,
    /// Shortens the Cat Cannon's recharge.
    CannonRecharge,
    /// Raises the Cat Cannon's damage.
    CannonPower,
    /// Raises the Worker Cat's money production.
    WorkerEfficiency,
    /// Raises the Worker Cat's wallet capacity.
    WalletCapacity,
    /// Raises the money an enemy drops when defeated.
    EnemyMoney,
    /// Raises the experience a cleared stage awards.
    Experience,
    /// Speeds up the recovery of Cat Energy outside battle.
    EnergyRecovery,
    /// Raises the Cat Energy cap outside battle.
    MaxEnergy,
    /// Raises every cat's health.
    CatHealth,
    /// Raises every cat's attack.
    CatAttack,
    /// Shortens every cat's recharge.
    CatRecharge,
    /// Strengthens abilities aimed at Red enemies.
    Red,
    /// Strengthens abilities aimed at Dark enemies.
    Dark,
    /// Strengthens abilities aimed at Floating enemies.
    Floating,
    /// Strengthens abilities aimed at Angel enemies.
    Angel,
    /// Weakens Alien enemies without a star.
    AlienWeakening,
    /// Offsets the recharge the Cat Cannon range upgrade costs.
    CannonRangeRecharge,
    /// Weakens Starred Alien enemies.
    StarredAlienWeakening,
    /// Strengthens abilities aimed at Metal enemies.
    Metal,
    /// Strengthens abilities aimed at Zombie enemies.
    Zombie,
    /// Strengthens abilities aimed at Alien enemies.
    Alien,
    /// Weakens enemies of the second Starred Alien type.
    StarredAlienType2,
    /// Weakens enemies of the third Starred Alien type.
    StarredAlienType3,
    /// Weakens enemies of the fourth Starred Alien type.
    StarredAlienType4,
    /// An effect identifier this crate does not recognize, carrying its raw value.
    Unknown(i32),
}

impl From<i32> for TreasureEffect {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::BaseHealth,
            1 => Self::CannonRecharge,
            2 => Self::CannonPower,
            3 => Self::WorkerEfficiency,
            4 => Self::WalletCapacity,
            5 => Self::EnemyMoney,
            6 => Self::Experience,
            7 => Self::EnergyRecovery,
            8 => Self::MaxEnergy,
            9 => Self::CatHealth,
            10 => Self::CatAttack,
            11 => Self::CatRecharge,
            12 => Self::Red,
            13 => Self::Dark,
            14 => Self::Floating,
            15 => Self::Angel,
            16 => Self::AlienWeakening,
            17 => Self::CannonRangeRecharge,
            18 => Self::StarredAlienWeakening,
            19 => Self::Metal,
            20 => Self::Zombie,
            21 => Self::Alien,
            22 => Self::StarredAlienType2,
            23 => Self::StarredAlienType3,
            24 => Self::StarredAlienType4,
            other => Self::Unknown(other),
        }
    }
}

impl TreasureEffect {
    /// Returns the identifier the effect column carries for this effect.
    ///
    /// # Returns
    /// An `i32` holding the identifier.
    pub fn id(self) -> i32 {
        match self {
            Self::BaseHealth => 0,
            Self::CannonRecharge => 1,
            Self::CannonPower => 2,
            Self::WorkerEfficiency => 3,
            Self::WalletCapacity => 4,
            Self::EnemyMoney => 5,
            Self::Experience => 6,
            Self::EnergyRecovery => 7,
            Self::MaxEnergy => 8,
            Self::CatHealth => 9,
            Self::CatAttack => 10,
            Self::CatRecharge => 11,
            Self::Red => 12,
            Self::Dark => 13,
            Self::Floating => 14,
            Self::Angel => 15,
            Self::AlienWeakening => 16,
            Self::CannonRangeRecharge => 17,
            Self::StarredAlienWeakening => 18,
            Self::Metal => 19,
            Self::Zombie => 20,
            Self::Alien => 21,
            Self::StarredAlienType2 => 22,
            Self::StarredAlienType3 => 23,
            Self::StarredAlienType4 => 24,
            Self::Unknown(value) => value,
        }
    }

    /// Returns the weakening effect that scales an enemy's health and attack.
    ///
    /// An enemy the player has not earned the matching treasures against is
    /// stronger by the part of the effect still missing.
    ///
    /// # Arguments
    /// * `trait_alien` - Whether the enemy carries the Alien trait.
    /// * `trait_starred_alien` - The enemy's Starred Alien column.
    ///
    /// # Returns
    /// An `Option` holding the effect, or `None` for an enemy no treasure weakens.
    pub fn for_enemy(trait_alien: bool, trait_starred_alien: i32) -> Option<Self> {
        match (trait_alien, trait_starred_alien) {
            (true, 0) => Some(Self::AlienWeakening),
            (true, 1) => Some(Self::StarredAlienWeakening),
            (_, 2) => Some(Self::StarredAlienType2),
            (_, 3) => Some(Self::StarredAlienType3),
            (_, 4) => Some(Self::StarredAlienType4),
            _ => None,
        }
    }
}

/// The strengthening a trait's treasures give a cat's abilities against that trait.
///
/// Carries the capped grade total of the trait's treasure effect, zero with none
/// held and three hundred with every chapter fully Superior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitBonus(pub i32);

impl TraitBonus {
    /// Returns the damage multiplier Strong Against deals.
    ///
    /// # Returns
    /// An `f64` from 1.5 with no treasures to 1.8 with all of them.
    pub fn strong_dealt(self) -> f64 {
        f64::from(1500 + self.0) / 1000.0
    }

    /// Returns the damage multiplier Strong Against takes.
    ///
    /// # Returns
    /// An `f64` from 0.5 with no treasures to 0.4 with all of them.
    pub fn strong_taken(self) -> f64 {
        f64::from(1500 - self.0) / 3000.0
    }

    /// Returns the damage multiplier Resistant takes.
    ///
    /// # Returns
    /// An `f64` from 0.25 with no treasures to 0.2 with all of them.
    pub fn resist_taken(self) -> f64 {
        f64::from(1500 - self.0) / 6000.0
    }

    /// Returns the damage multiplier Massive Damage deals.
    ///
    /// # Returns
    /// An `f64` from 3 with no treasures to 4 with all of them.
    pub fn massive_dealt(self) -> f64 {
        f64::from(900 + self.0) / 300.0
    }

    /// Returns the damage multiplier Insane Damage deals.
    ///
    /// # Returns
    /// An `f64` from 5 with no treasures to 6 with all of them.
    pub fn insane_dealt(self) -> f64 {
        f64::from(1500 + self.0) / 300.0
    }

    /// Returns the damage multiplier Insanely Tough takes.
    ///
    /// # Returns
    /// An `f64` from one sixth with no treasures to one seventh with all of them.
    pub fn insanely_tough_taken(self) -> f64 {
        f64::from(2100 - self.0) / 12600.0
    }

    /// Returns the multiplier on Freeze, Slow, Weaken and Curse durations, and on a cat's Dodge.
    ///
    /// # Returns
    /// An `f64` from 1 with no treasures to 1.2 with all of them.
    pub fn duration(self) -> f64 {
        f64::from(1500 + self.0) / 1500.0
    }

    /// Returns a Freeze, Slow, Weaken or Curse duration a cat inflicts, or a cat's Dodge duration, after the treasures.
    ///
    /// The engine scales the frame count in integers and truncates, so a
    /// duration that does not divide evenly comes out one frame shorter than
    /// the multiplier alone suggests.
    ///
    /// # Arguments
    /// * `frames` - The duration the unit's combat row declares.
    ///
    /// # Returns
    /// An `i32` holding the duration in frames.
    pub fn scale_duration(self, frames: i32) -> i32 {
        (i64::from(frames) * i64::from(1500 + self.0) / 1500) as i32
    }
}

/// Returns the money an enemy drops before combos and bounty doubling.
///
/// # Arguments
/// * `drop` - The enemy's money column.
/// * `accounting` - The Accounting upgrade as the engine reads it, zero for level one, with plus levels added.
/// * `treasure` - The value of the enemy money effect.
///
/// # Returns
/// An `i32` holding the money, never below zero.
pub fn enemy_money(drop: i32, accounting: i32, treasure: i32) -> i32 {
    let scaled = i64::from(accounting * 5 + treasure + 100) * i64::from(drop) / 100;

    scaled.max(0) as i32
}

/// Returns a cat's recharge before combos and stage rules.
///
/// # Arguments
/// * `cooldown` - The cat's recharge as the engine reads it.
/// * `research` - The Research upgrade as the engine reads it, zero for level one, with plus levels added.
/// * `treasure` - The value of the cat recharge effect.
///
/// # Returns
/// An `i32` holding the recharge in frames, never below sixty.
pub fn unit_recharge(cooldown: i32, research: i32, treasure: i32) -> i32 {
    let reduced = cooldown - (treasure + research * 6);

    if reduced > 60 { reduced } else { 60 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_identifier_round_trips() {
        for id in 0..=24 {
            assert_eq!(TreasureEffect::from(id).id(), id);
        }

        assert_eq!(TreasureEffect::from(99), TreasureEffect::Unknown(99));
    }

    #[test]
    fn a_full_set_of_fruit_lands_on_the_familiar_multipliers() {
        let bonus = TraitBonus(300);

        assert_eq!(bonus.strong_dealt(), 1.8);
        assert_eq!(bonus.massive_dealt(), 4.0);
        assert_eq!(bonus.resist_taken(), 0.2);
        assert_eq!(TraitBonus(0).insanely_tough_taken(), 1.0 / 6.0);
    }

    #[test]
    fn a_duration_truncates_the_way_the_engine_does() {
        assert_eq!(TraitBonus(300).scale_duration(150), 180);
        assert_eq!(TraitBonus(300).scale_duration(38), 45);
        assert_eq!(TraitBonus(0).scale_duration(38), 38);
    }

    #[test]
    fn maxed_accounting_and_treasure_pay_three_point_nine_five() {
        assert_eq!(enemy_money(100, 29, 150), 395);
        assert_eq!(unit_recharge(400, 29, 90), 136);
        assert_eq!(unit_recharge(100, 29, 90), 60);
    }
}
