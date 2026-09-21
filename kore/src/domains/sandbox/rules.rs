use nyanko::chapter::map::RuleType;
use nyanko::chapter::stage::CharaGroupType;
use nyanko::chapter::{Map, Stage};

const FOUR_CROWN: i8 = 3;
const FOUR_CROWN_MASK: u8 = 6;
const LEGEND_TIER: i32 = 3;
const TOP_ROW: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    rarity_mask: u8,
    single_row: bool,
    min_cost: i32,
    max_cost: i32,
    only: Option<Vec<u32>>,
    never: Vec<u32>,
    flat_cost: Option<i32>,
    cost_multiplier: i32,
    cost_percent: Vec<u32>,
    lineup_caps: Vec<u32>,
    banned_effects: Vec<u32>,
}

impl Rules {
    pub fn of(stage: &Stage, map: Option<&Map>, crown: i8) -> Self {
        let mut rules = Self::default();

        if let Some(map) = map {
            rules.banned_effects.clone_from(&map.invalid_combos);
            rules.cost_multiplier = i32::try_from(map.cost_multiplier).unwrap_or(0);

            for rule in map.special_rules.iter().flat_map(|held| held.rules.iter()) {
                match rule {
                    RuleType::CheapLabor(params) => rules.flat_cost = params.first().and_then(|cost| i32::try_from(*cost).ok()),
                    RuleType::CatCost(params) => rules.cost_percent.clone_from(params),
                    RuleType::LineupLimit(params) => rules.lineup_caps.clone_from(params),
                    _ => (),
                }
            }
        }

        let highest = if stage.max_crowns == 0 { 1 } else { stage.max_crowns as i8 };
        let aimed = stage.target_crowns == -1 || stage.target_crowns == crown;

        if !aimed || stage.target_crowns >= highest {
            return rules;
        }

        rules.rarity_mask = if crown == FOUR_CROWN { FOUR_CROWN_MASK } else { stage.rarity_mask };
        rules.single_row = stage.allowed_rows == 1;
        rules.min_cost = i32::try_from(stage.min_cost).unwrap_or(i32::MAX);
        rules.max_cost = i32::try_from(stage.max_cost).unwrap_or(i32::MAX);

        if let Some(group) = &stage.charagroup {
            match group.kind {
                CharaGroupType::OnlyUse => rules.only = Some(group.units.clone()),
                CharaGroupType::CannotUse => rules.never.clone_from(&group.units),
                _ => (),
            }
        }

        rules
    }

    pub fn cost(&self, base: i32, rarity: usize) -> i32 {
        if let Some(flat) = self.flat_cost {
            return flat;
        }

        let cost = match self.cost_multiplier {
            0 => base.max(0) * LEGEND_TIER / 2,
            multiplier => base.max(0) * multiplier / 100,
        };

        self.cost_percent
            .get(rarity)
            .and_then(|percent| i32::try_from(*percent).ok())
            .map_or(cost, |percent| cost * percent / 100)
    }

    pub fn bars(&self, unit: u32, rarity: usize, cost: i32, earlier: usize, slot: Option<usize>) -> bool {
        if self.single_row && slot.is_some_and(|slot| slot >= TOP_ROW) {
            return true;
        }

        if self.rarity_mask != 0 && (u32::from(self.rarity_mask) >> (rarity & 0x1f)) & 1 == 0 {
            return true;
        }

        let (least, most) = (self.min_cost, self.max_cost);
        let priced_out = match (least != 0, most != 0) {
            (true, true) if least == most => cost != least,
            (true, true) if least > most => cost < least && cost > most,
            (true, true) => cost < least || cost > most,
            (true, false) => cost < least,
            (false, true) => cost > most,
            (false, false) => false,
        };

        if priced_out || self.never.contains(&unit) || self.only.as_ref().is_some_and(|allowed| !allowed.contains(&unit)) {
            return true;
        }

        self.lineup_caps.get(rarity).is_some_and(|cap| earlier >= *cap as usize)
    }

    pub fn bans(&self, effect: i32) -> bool {
        u32::try_from(effect).is_ok_and(|effect| self.banned_effects.contains(&effect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A flat-cost map replaces the price outright; a percent map scales the Legend price per rarity.
    #[test]
    fn a_map_rule_reprices_a_unit() {
        let flat = Rules { flat_cost: Some(500), ..Rules::default() };
        let scaled = Rules { cost_percent: vec![100, 100, 100, 50, 100, 100], ..Rules::default() };

        assert_eq!(Rules::default().cost(100, 0), 150);
        assert_eq!(flat.cost(100, 3), 500);
        assert_eq!(scaled.cost(100, 3), 75);
        assert_eq!(scaled.cost(100, 4), 150);
        assert_eq!(Rules { cost_multiplier: 100, ..Rules::default() }.cost(100, 0), 100, "a map multiplier replaces the Legend price");
    }

    #[test]
    fn a_restriction_bars_the_units_it_names() {
        let rules = Rules { rarity_mask: 0b100, max_cost: 1200, never: vec![7], lineup_caps: vec![9, 9, 1, 9, 9, 9], ..Rules::default() };

        assert!(rules.bars(1, 4, 100, 0, Some(0)), "an Uber is outside a Rare-only mask");
        assert!(rules.bars(1, 2, 1500, 0, Some(0)), "priced over the cap");
        assert!(rules.bars(7, 2, 100, 0, Some(0)), "named by a Cannot Use group");
        assert!(rules.bars(1, 2, 100, 1, Some(1)), "the second Rare is over a cap of one");
        assert!(!rules.bars(1, 2, 100, 0, Some(0)));

        let one_row = Rules { single_row: true, ..Rules::default() };

        assert!(one_row.bars(1, 2, 100, 0, Some(5)), "the bottom row cannot deploy");
        assert!(!one_row.bars(1, 2, 100, 0, Some(4)));
        assert!(!one_row.bars(1, 2, 100, 0, None), "the bench is not a row");
    }
}
