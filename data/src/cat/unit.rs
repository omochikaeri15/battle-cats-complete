//! Cat unit data: the aggregate [`Unit`], the tables it draws on, the parsers
//! behind them.

mod equipmentlist;
mod equipmentslot;
mod nyancombo;
mod nyancombodata;
mod nyancombofilter;
mod nyancomboparam;
mod skillacquisition;
mod skilldescriptions;
mod skilllevel;
mod unitbuy;
mod unitevolve;
mod unitexplanation;
mod unitlevel;

pub mod unitid;

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::combat::{Entity, EntityError};
use crate::common::file::Separator;

pub use equipmentlist::{Equipment, EquipmentListError};
pub use equipmentslot::{EquipmentSlot, EquipmentSlotError};
pub use nyancombo::{Nyancombo, NyancomboError};
pub use nyancombodata::{ComboSlot, ComboStrength, NyancomboData, NyancomboDataError};
pub use nyancombofilter::{NyancomboFilter, NyancomboFilterError};
pub use nyancomboparam::{NyancomboParam, NyancomboParamError};
pub use skillacquisition::{SkillAcquisitionError, Talent, TalentGroup};
pub use skilldescriptions::{SkillDescriptions, SkillDescriptionsError};
pub use skilllevel::{SkillLevelError, TalentCost};
pub use unitbuy::{EvolveMaterial, UnitBuy, UnitBuyError};
pub use unitevolve::{UnitEvolve, UnitEvolveError};
pub use unitexplanation::{UnitExplanation, UnitExplanationError};
pub use unitlevel::{LevelCurve, LevelError};

/// Represents errors that can occur while aggregating a unit from its sources.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssembleError {
    /// The progression table carried no row for the requested unit identifier.
    MissingUnitBuy(u32),
    /// The unit's combat file could not be parsed into any usable form.
    InvalidCombat(u32, EntityError),
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingUnitBuy(id) => {
                write!(f, "The unit buy table contained no row for unit {id}.")
            }
            Self::InvalidCombat(id, source) => {
                write!(f, "The combat file for unit {id} could not be parsed: {source}")
            }
        }
    }
}

impl std::error::Error for AssembleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingUnitBuy(_) => None,
            Self::InvalidCombat(_, source) => Some(source),
        }
    }
}

/// Borrowed references to the shared tables required to aggregate any unit.
///
/// Lets a caller parse each whole-roster table once and borrow it across every
/// unit, rather than reparsing or cloning per unit.
///
/// All six must describe the same game version and region, since the identifiers
/// they are keyed by are only meaningful within one such set.
#[derive(Debug, Clone, Copy)]
pub struct Tables<'a> {
    /// The progression table, keyed by unit identifier.
    pub unitbuy: &'a HashMap<u32, UnitBuy>,
    /// The level growth table, keyed by unit identifier.
    pub curves: &'a HashMap<u32, LevelCurve>,
    /// The talent configuration table, keyed by unit identifier.
    pub talents: &'a HashMap<u32, Talent>,
    /// The evolution text table, keyed by unit identifier.
    pub evolve: &'a HashMap<u32, UnitEvolve>,
    /// The talent cost table, keyed by cost identifier.
    pub talent_costs: &'a HashMap<u8, TalentCost>,
    /// The skill description table, indexed by skill identifier.
    pub skill_descriptions: &'a [String],
}

/// The fully-aggregated representation of a Cat unit.
///
/// The engine scatters a unit's definition across several localized and
/// mechanical files; this combines them into one payload suited to being
/// serialized as the pipeline's final output. Build one with [`Unit::assemble`]
/// rather than populating the fields directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// The base mathematical identifier for the unit, corresponding to its directory and file prefixes.
    pub id: u32,
    /// A fixed array containing the unit's display names across all 4 potential evolutionary forms.
    /// Evaluates to `None` if the form does not exist or lacks a unique name.
    pub names: [Option<String>; 4],
    /// A fixed array containing the multi-line flavor text and lore explanations for all 4 forms.
    /// Evaluates to `None` if the form does not exist or lacks unique text.
    pub descriptions: [Option<Vec<String>>; 4],
    /// The raw mechanical combat data, hitboxes, and active abilities, mapped strictly to each of the 4 forms.
    /// Missing forms evaluate to `None`.
    pub combat: [Option<Entity>; 4],
    /// The absolute duration of the primary attack animation, parsed chronologically from the `maanim` sequence data.
    ///
    /// Produced by `graphics::rig::Animation::scan_length`, which requires the
    /// non-default `graphics` feature; leave `None` without it.
    pub attack_frames: [Option<i32>; 4],
    /// The mathematical progression curve dictating the required XP to reach subsequent levels.
    pub level_curve: Option<LevelCurve>,
    /// The collection of unlockable NP upgrades and abilities for the unit's True/Ultra forms.
    pub talents: Option<Talent>,
    /// The financial and progression prerequisites, including deployment cost, cooldown frames, and unlock conditions.
    ///
    /// Egg counterparts are reached through [`UnitBuy::egg_ids`].
    pub unitbuy: UnitBuy,
    /// Localized instructional text detailing the specific items and XP required to evolve the unit.
    pub evolve_text: [Option<Vec<String>>; 4],
    /// A deterministically sorted dictionary associating specific, raw talent integer IDs with their escalating NP upgrade costs.
    ///
    /// Only the curves this unit's talent groups reference are retained.
    pub talent_costs: BTreeMap<u8, TalentCost>,
    /// Localized, human-readable descriptions explaining the mechanical function of the unit's assigned skills.
    ///
    /// Positionally aligned with the groups of [`Unit::talents`]. A group with no
    /// resolvable description contributes an empty string, preserving alignment.
    pub skill_descriptions: Vec<String>,
}

impl Unit {
    /// Aggregates a single unit from its own files and the shared roster tables.
    ///
    /// This performs every lookup, key conversion, and sentinel decision the
    /// engine's layout requires.
    ///
    /// Absent localized text is a data condition rather than a failure, and
    /// yields empty name and description arrays. Only a missing progression row
    /// or an unparseable combat file prevent aggregation.
    ///
    /// # Arguments
    /// * `id` - The unit's internal identifier, used to key every table lookup.
    /// * `stats` - The raw bytes of the unit's own `unit<id>.csv` combat file.
    /// * `stats_separator` - The delimiter the combat file is written with, or `None` to detect it from the content.
    /// * `explanation` - The raw bytes of the unit's own localized explanation file.
    /// * `explanation_separator` - The delimiter the explanation file is written with, or `None` to detect it from the content.
    /// * `attack_frames` - The measured attack animation length for each of the 4 forms, which the caller obtains from the `graphics` feature or leaves as `None`.
    /// * `tables` - Borrowed references to the shared roster tables.
    ///
    /// # Returns
    /// A `Result` containing the fully aggregated `Unit` on success, or an
    /// `AssembleError` identifying the unit and the source that prevented it.
    pub fn assemble(
        id: u32,
        stats: impl AsRef<[u8]>,
        stats_separator: Option<Separator>,
        explanation: impl AsRef<[u8]>,
        explanation_separator: Option<Separator>,
        attack_frames: [Option<i32>; 4],
        tables: &Tables<'_>,
    ) -> Result<Self, AssembleError> {
        Self::assemble_inner(
            id,
            stats.as_ref(),
            stats_separator,
            explanation.as_ref(),
            explanation_separator,
            attack_frames,
            tables,
        )
    }

    fn assemble_inner(
        id: u32,
        stats: &[u8],
        stats_separator: Option<Separator>,
        explanation: &[u8],
        explanation_separator: Option<Separator>,
        attack_frames: [Option<i32>; 4],
        tables: &Tables<'_>,
    ) -> Result<Self, AssembleError> {
        let Some(unitbuy) = tables.unitbuy.get(&id) else {
            return Err(AssembleError::MissingUnitBuy(id));
        };

        let rows = unitid::parse(stats, stats_separator).map_err(|source| AssembleError::InvalidCombat(id, source))?;

        let mut combat: [Option<Entity>; 4] = [const { None }; 4];
        for (slot, row) in combat.iter_mut().zip(rows) {
            *slot = Some(row);
        }

        let (names, descriptions) = UnitExplanation::parse(explanation, explanation_separator)
            .map_or_else(|_| Default::default(), |text| (text.names, text.descriptions));

        let talents = tables.talents.get(&id).cloned();

        let mut talent_costs = BTreeMap::new();
        let mut skill_descriptions = Vec::new();

        if let Some(configuration) = &talents {
            for group in &configuration.groups {
                if let Some(costs) = tables.talent_costs.get(&group.cost_id) {
                    talent_costs.insert(group.cost_id, costs.clone());
                }

                let text = tables
                    .skill_descriptions
                    .get(usize::from(group.text_id))
                    .cloned()
                    .unwrap_or_default();
                skill_descriptions.push(text);
            }
        }

        Ok(Self {
            id,
            names,
            descriptions,
            combat,
            attack_frames,
            level_curve: tables.curves.get(&id).cloned(),
            talents,
            unitbuy: unitbuy.clone(),
            evolve_text: tables.evolve.get(&id).map_or_else(Default::default, |row| row.texts.clone()),
            talent_costs,
            skill_descriptions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables<'a>(
        unitbuy: &'a HashMap<u32, UnitBuy>,
        curves: &'a HashMap<u32, LevelCurve>,
        talents: &'a HashMap<u32, Talent>,
        evolve: &'a HashMap<u32, UnitEvolve>,
        costs: &'a HashMap<u8, TalentCost>,
        descriptions: &'a [String],
    ) -> Tables<'a> {
        Tables {
            unitbuy,
            curves,
            talents,
            evolve,
            talent_costs: costs,
            skill_descriptions: descriptions,
        }
    }

    fn group(cost_id: u8, text_id: u8) -> TalentGroup {
        TalentGroup { ability_id: 1, cost_id, text_id, ..Default::default() }
    }

    #[test]
    fn assemble_fills_forms_and_tables() {
        let stats = b"100,1,10,50,20,300,75,60,0,120\n200,2,10,90,20,300,75,60,0,120\n";
        let explanation = "Cat,Just a cat\nTank Cat,A sturdier cat";

        let mut unitbuy = HashMap::new();
        unitbuy.insert(7, UnitBuy { rarity: 3, egg_id_normal: 12, egg_id_evolved: -1, ..Default::default() });

        let mut curves = HashMap::new();
        curves.insert(7, LevelCurve { increments: vec![20, 10] });

        let mut talents = HashMap::new();
        talents.insert(7, Talent { id: 7, type_id: 1, groups: vec![group(2, 1), group(9, 0)] });

        let mut evolve = HashMap::new();
        evolve.insert(7, UnitEvolve { texts: [None, None, Some(vec!["Needs a Purple Fruit".into()]), None] });

        let mut costs = HashMap::new();
        costs.insert(2, TalentCost { costs: vec![10, 20] });
        costs.insert(5, TalentCost { costs: vec![99] });

        let descriptions = vec!["Attack up".to_string(), "Cost down".to_string()];

        let unit = Unit::assemble(
            7,
            stats,
            None,
            explanation,
            None,
            [Some(42), None, None, None],
            &tables(&unitbuy, &curves, &talents, &evolve, &costs, &descriptions),
        )
        .unwrap();

        assert_eq!(unit.id, 7);
        assert_eq!(unit.names[0].as_deref(), Some("Cat"));
        assert_eq!(unit.names[1].as_deref(), Some("Tank Cat"));
        assert!(unit.combat[0].is_some() && unit.combat[1].is_some());
        assert!(unit.combat[2].is_none() && unit.combat[3].is_none());
        assert_eq!(unit.attack_frames, [Some(42), None, None, None]);
        assert_eq!(unit.unitbuy.rarity, 3);
        assert_eq!(unit.level_curve.as_ref().unwrap().increments, vec![20, 10]);
        assert_eq!(unit.evolve_text[2].as_ref().unwrap()[0], "Needs a Purple Fruit");

        assert_eq!(unit.talent_costs.keys().copied().collect::<Vec<_>>(), vec![2]);
        assert_eq!(unit.skill_descriptions, vec!["Cost down".to_string(), "Attack up".to_string()]);
    }

    #[test]
    fn assemble_reports_the_offending_unit() {
        let empty_buy = HashMap::new();
        let curves = HashMap::new();
        let talents = HashMap::new();
        let evolve = HashMap::new();
        let costs = HashMap::new();
        let descriptions: Vec<String> = Vec::new();
        let set = tables(&empty_buy, &curves, &talents, &evolve, &costs, &descriptions);

        let outcome = Unit::assemble(404, b"100,1,10,50,20,300,75,60,0,120", None, "", None, [None; 4], &set);
        assert_eq!(outcome.unwrap_err(), AssembleError::MissingUnitBuy(404));

        let mut unitbuy = HashMap::new();
        unitbuy.insert(404, UnitBuy::default());
        let set = tables(&unitbuy, &curves, &talents, &evolve, &costs, &descriptions);

        let outcome = Unit::assemble(404, b"", None, "", None, [None; 4], &set);
        assert_eq!(outcome.unwrap_err(), AssembleError::InvalidCombat(404, EntityError::EmptyFile));
    }

    #[test]
    fn assemble_tolerates_absent_localized_text() {
        let mut unitbuy = HashMap::new();
        unitbuy.insert(1, UnitBuy::default());
        let curves = HashMap::new();
        let talents = HashMap::new();
        let evolve = HashMap::new();
        let costs = HashMap::new();
        let descriptions: Vec<String> = Vec::new();

        let unit = Unit::assemble(
            1,
            b"100,1,10,50,20,300,75,60,0,120",
            None,
            "",
            None,
            [None; 4],
            &tables(&unitbuy, &curves, &talents, &evolve, &costs, &descriptions),
        )
        .unwrap();

        assert_eq!(unit.names, [const { None }; 4]);
        assert!(unit.talents.is_none());
        assert!(unit.skill_descriptions.is_empty());
    }

    #[test]
    fn egg_ids_resolve_each_column_independently() {
        let both = UnitBuy { egg_id_normal: 4, egg_id_evolved: 9, ..Default::default() };
        assert_eq!(both.egg_ids(), (Some(4), Some(9)));

        let mixed = UnitBuy { egg_id_normal: 4, egg_id_evolved: -1, ..Default::default() };
        assert_eq!(mixed.egg_ids(), (Some(4), None));

        let flipped = UnitBuy { egg_id_normal: -1, egg_id_evolved: 9, ..Default::default() };
        assert_eq!(flipped.egg_ids(), (None, Some(9)));

        let neither = UnitBuy::default();
        assert_eq!(neither.egg_ids(), (None, None));
    }

    #[test]
    fn blank_lines_do_not_shift_table_keys() {
        let curves = LevelCurve::parse("10,10\n\n30,30\n", None).unwrap();
        assert_eq!(curves[&0].increments, vec![10, 10]);
        assert!(!curves.contains_key(&1));
        assert_eq!(curves[&2].increments, vec![30, 30]);

        let buys = UnitBuy::parse("0,100\n\n0,300\n", None).unwrap();
        assert_eq!(buys[&0].purchase_cost, 100);
        assert_eq!(buys[&2].purchase_cost, 300);
    }
}
