use std::borrow::Cow;
use std::collections::HashMap;

use nyanko::combat::{get_ability, AttributeIndex, DictionaryIndex, Faction, Glossary, Identity, TraitLabels};
use nyanko::files::Localizable;

use crate::systems::combat::registry::get_display_def;

const TALENT_KEY: &str = "potential_skill_name";
const BONUS_KEY: &str = "BonusNameLabel";
const BONUS_KINDS: usize = 8;

#[derive(Debug, Default)]
pub struct NameBook {
    cat: Option<Glossary>,
    enemy: Option<Glossary>,
    traits: Option<TraitLabels>,
    index: Option<DictionaryIndex>,
    attributes: Option<AttributeIndex>,
    talents: HashMap<u8, String>,
    bonuses: Vec<Option<String>>,
    cannons: HashMap<i32, String>,
}

impl NameBook {
    pub fn new(cat: Option<Glossary>, enemy: Option<Glossary>, traits: Option<TraitLabels>, index: Option<DictionaryIndex>, attributes: Option<AttributeIndex>, cannons: HashMap<i32, String>, localizable: &Localizable) -> Self {
        let talents = (1..=u8::MAX)
            .filter_map(|id| {
                let name = localizable.lookup(&format!("{TALENT_KEY}{id:02}"))?.trim();

                (!name.is_empty()).then(|| (id, name.to_owned()))
            })
            .collect();
        let bonuses = (0..BONUS_KINDS)
            .map(|index| localizable.lookup(&format!("{BONUS_KEY}{index:03}")).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned))
            .collect();

        Self { cat, enemy, traits, index, attributes, talents, bonuses, cannons }
    }

    fn cat_line(&self, ability: &nyanko::combat::Ability) -> Option<u8> {
        self.index
            .as_ref()
            .zip(self.attributes.as_ref())
            .zip(ability.dictionary_id)
            .and_then(|((index, attributes), id)| index.glossary_line(id, attributes))
            .or(ability.cat_glossary)
    }

    fn glossary(&self, identity: Identity, faction: Faction) -> Option<&str> {
        let ability = get_ability(identity)?;
        let cat = || self.cat.as_ref().zip(self.cat_line(ability)).and_then(|(glossary, line)| glossary.name(line));
        let enemy = || self.enemy.as_ref().zip(ability.enemy_glossary).and_then(|(glossary, line)| glossary.name(line));

        match faction {
            Faction::Cat => cat().or_else(enemy),
            Faction::Enemy => enemy().or_else(cat),
        }
    }

    pub fn glossary_line(&self, identity: Identity, faction: Faction) -> Option<u8> {
        let ability = get_ability(identity)?;

        match faction {
            Faction::Cat => self.cat_line(ability),
            Faction::Enemy => ability.enemy_glossary,
        }
    }

    pub fn ability(&self, identity: Identity, faction: Faction) -> &str {
        self.glossary(identity, faction).unwrap_or(get_display_def(identity).name)
    }

    pub fn description(&self, identity: Identity, faction: Faction) -> Option<&[String]> {
        let ability = get_ability(identity)?;
        let (glossary, line) = match faction {
            Faction::Cat => self.cat.as_ref().zip(self.cat_line(ability)),
            Faction::Enemy => self.enemy.as_ref().zip(ability.enemy_glossary),
        }?;
        let lines = glossary.entry(line)?.lines.as_slice();

        (!lines.is_empty()).then_some(lines)
    }

    fn orb_label(&self, identity: Identity) -> Option<&nyanko::combat::TraitLabel> {
        let row = get_ability(identity)?.orb_trait?;

        self.traits.as_ref()?.entries.get(usize::from(row)).filter(|label| !label.name.is_empty())
    }

    pub fn trait_label(&self, identity: Identity) -> &str {
        self.orb_label(identity)
            .map(|label| label.name.as_str())
            .or_else(|| self.glossary(identity, Faction::Enemy))
            .unwrap_or(get_display_def(identity).name)
    }

    pub fn trait_plural(&self, identity: Identity) -> Cow<'_, str> {
        match self.orb_label(identity).filter(|label| !label.plural.is_empty()) {
            Some(label) => Cow::Borrowed(label.plural.as_str()),
            None => Cow::Owned(format!("{} Enemies", self.trait_label(identity))),
        }
    }

    pub fn talent(&self, id: u8) -> Option<&str> {
        self.talents.get(&id).map(String::as_str)
    }

    pub fn bonus(&self, index: usize) -> Option<&str> {
        self.bonuses.get(index)?.as_deref()
    }

    pub fn cannon(&self, part: i32) -> Option<&str> {
        self.cannons.get(&part).map(String::as_str)
    }
}




