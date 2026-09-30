use std::collections::HashMap;

use nyanko::combat::{AttributeIndex, DictionaryIndex, Glossary, TraitLabels};
use nyanko::files::Localizable;

use crate::common::region;
use crate::systems::combat::NameBook;
use crate::Vfs;

use super::Slot;

const CAT_GLOSSARY: &str = "nyankoPictureBook2.csv";
const ENEMY_GLOSSARY: &str = "EnemyPictureBook2.csv";
const TRAIT_LABELS: &str = "attribute_explonation.tsv";
const CANNON_NAMES: &str = "CastleRecipeDescriptions.csv";
const DICTIONARY_INDEX: &str = "nyankoPictureBookData_EffectAbility.csv";
const ATTRIBUTE_INDEX: &str = "nyankoPictureBookData_Attribute.csv";
const PREFIXES: [&str; 5] = ["nyankoPictureBook2", "EnemyPictureBook2", "attribute_explonation", "CastleRecipeDescriptions", "nyankoPictureBookData_"];
const BASE_ROW: i32 = 0;

#[derive(Default)]
pub struct NameStore {
    cat: Slot<Option<Glossary>>,
    enemy: Slot<Option<Glossary>>,
    traits: Slot<Option<TraitLabels>>,
    index: Slot<Option<DictionaryIndex>>,
    attributes: Slot<Option<AttributeIndex>>,
    cannons: Slot<HashMap<i32, String>>,
}

impl NameStore {
    pub fn book(&self, vfs: &Vfs, localizable: &Localizable) -> NameBook {
        let cat = super::cached(&self.cat, || glossary(vfs, CAT_GLOSSARY));
        let enemy = super::cached(&self.enemy, || glossary(vfs, ENEMY_GLOSSARY));
        let traits = super::cached(&self.traits, || {
            super::layered(vfs, TRAIT_LABELS).into_iter().find_map(|bytes| TraitLabels::parse(bytes).ok())
        });

        let index = super::cached(&self.index, || super::parsed(vfs, DICTIONARY_INDEX, DictionaryIndex::parse));
        let attributes = super::cached(&self.attributes, || super::parsed(vfs, ATTRIBUTE_INDEX, AttributeIndex::parse));
        let cannons = super::cached(&self.cannons, || cannon_names(vfs));

        NameBook::new((*cat).clone(), (*enemy).clone(), (*traits).clone(), (*index).clone(), (*attributes).clone(), (*cannons).clone(), localizable)
    }

    pub(super) fn evict(&self, filename: &str) {
        if PREFIXES.iter().any(|prefix| filename.starts_with(prefix)) {
            self.clear();
        }
    }

    pub(super) fn clear(&self) {
        super::reset(&self.cat);
        super::reset(&self.enemy);
        super::reset(&self.traits);
        super::reset(&self.index);
        super::reset(&self.attributes);
        super::reset(&self.cannons);
    }
}

fn cannon_names(vfs: &Vfs) -> HashMap<i32, String> {
    let mut names: HashMap<i32, String> = HashMap::new();

    for (name, bytes) in super::named(vfs, CANNON_NAMES) {
        let delimiter = region::text_separator(&name).char();
        let content = String::from_utf8_lossy(&bytes);

        for line in content.lines() {
            let mut cells = line.split(delimiter).map(str::trim);
            let Some(part) = cells.next().and_then(|cell| cell.parse::<i32>().ok()).filter(|part| *part != BASE_ROW) else {
                continue;
            };
            let Some(label) = cells.next().filter(|cell| !cell.is_empty()) else {
                continue;
            };

            names.entry(part).or_insert_with(|| label.to_owned());
        }
    }

    names
}

fn glossary(vfs: &Vfs, filename: &str) -> Option<Glossary> {
    let mut merged: Option<Glossary> = None;

    for (name, bytes) in super::named(vfs, filename) {
        let Ok(layer) = Glossary::parse(bytes, Some(region::text_separator(&name))) else {
            continue;
        };

        let Some(held) = merged.as_mut() else {
            merged = Some(layer);
            continue;
        };

        if layer.entries.len() > held.entries.len() {
            held.entries.resize_with(layer.entries.len(), Default::default);
        }

        for (entry, filler) in held.entries.iter_mut().zip(layer.entries) {
            if entry.name.is_none() && entry.lines.is_empty() {
                *entry = filler;
            }
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use super::*;

    // The base's own row is not a cannon, and a name the first language leaves blank
    // has to fall through to the next one.
    #[test]
    fn cannon_names_skip_the_base_row_and_layer_languages() {
        let root = env::temp_dir().join(format!("bcc-cannon-names-{}", std::process::id()));
        fs::create_dir_all(&root).expect("scratch dir");
        fs::write(root.join("CastleRecipeDescriptions_en.csv"), "0|Enhance Base|x\n1|Slow Beam|y\n2||z\n").expect("seed en");
        fs::write(root.join("CastleRecipeDescriptions_ja.csv"), "0,城強化,x\n1,スロウ砲,y\n2,鉄壁砲,z\n").expect("seed ja");

        let vfs = Vfs::with_priority(&[String::new(), "en".to_string(), "ja".to_string(), "--".to_string()]);
        vfs.create(root.as_path()).expect("mount the scratch dir");

        let names = cannon_names(&vfs);

        assert_eq!(names.get(&0), None);
        assert_eq!(names.get(&1).map(String::as_str), Some("Slow Beam"));
        assert_eq!(names.get(&2).map(String::as_str), Some("鉄壁砲"));

        let _ = fs::remove_dir_all(&root);
    }
}


