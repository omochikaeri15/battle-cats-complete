mod footage;
mod save;
mod tape;
#[cfg(all(feature = "disk", feature = "mods"))]
mod disk;

use crate::common::architecture;
use crate::domains::cat::scanner::{self, CatEntry};
use crate::domains::settings::ScannerConfig;
use crate::Vault;

#[cfg(all(feature = "disk", feature = "mods"))]
pub use disk::{Bundle, Recording, delete, export, index, inspect_bundle, inspect_dir, latest, library, list, next_bundle, pack, read_faults, read_input, read_save, rename, restamp, stage, stamp};
pub use footage::{Footage, Opened, Packer};
pub use save::{God, Level, Options, Parts, Save, Screen, Seeds, Setup, Stage, Tutorial, Unit, Variant};
pub use tape::{Cue, Key, format_frame, parse};

pub const EXTENSION: &str = "bcv";
pub const VANILLA_APP: &str = "The Battle Cats";

const SAVE: &str = "save";
const INPUT: &str = "input";
const FAULT: &str = "fault";
const MANIFEST: &str = "manifest";
const ASSETS: &str = architecture::REPLAY;
const BASE_FORMS: usize = 2;
const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const UNIT_FILE_PREFIX: &str = "unit";
const UNIT_FILE_SUFFIX: &str = ".csv";
const EXPLANATION_PREFIX: &str = "Unit_Explanation";
const LANGUAGE_SEPARATOR: char = '_';

pub struct Staged {
    pub summary: Summary,
    pub vault: Vault,
    pub cats: Vec<CatEntry>,
}

pub fn staged(summary: Summary, vault: Vault, config: &ScannerConfig) -> Staged {
    let mut units: Vec<u32> = summary
        .save
        .iter()
        .flat_map(|save| save.setup.lineup.iter())
        .filter_map(|unit| u32::try_from(unit.unit).ok())
        .collect();

    units.dedup();

    let lenient = ScannerConfig { show_invalid_cats: true, ..config.clone() };
    let mut cats: Vec<CatEntry> = units.into_iter().filter_map(|id| scanner::scan_single(id, &vault, &lenient)).collect();

    for unit in summary.save.iter().flat_map(|save| save.setup.lineup.iter()) {
        let fielded = cats
            .iter_mut()
            .find(|cat| i32::try_from(cat.id).is_ok_and(|id| id == unit.unit))
            .zip(usize::try_from(unit.form).ok())
            .and_then(|(cat, form)| cat.forms.get_mut(form));

        if let Some(present) = fielded {
            *present = true;
        }
    }

    for cat in &mut cats {
        for form in 0..BASE_FORMS {
            cat.forms[form] |= cat.stats[form].is_some();
        }

        for form in 0..cat.forms.len() {
            if cat.forms[form] && cat.deploy_icon_paths[form].is_none() {
                cat.deploy_icon_paths[form] = scanner::deploy_icon(&vault.vfs, cat, form);
            }
        }
    }

    Staged { summary, vault, cats }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forgiven {
    pub frame: usize,
    pub site: String,
    pub reason: String,
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(glyph) = chars.next() {
        if glyph != '\\' {
            out.push(glyph);

            continue;
        }

        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    out
}

fn parse_faults(text: &str) -> Vec<Forgiven> {
    text.lines()
        .filter_map(|line| {
            let (frame, rest) = line.split_once('\t')?;
            let (site, reason) = rest.split_once('\t')?;

            Some(Forgiven { frame: frame.parse().ok()?, site: site.to_owned(), reason: unescape(reason) })
        })
        .collect()
}

fn bundled(name: &str) -> bool {
    name.strip_prefix(ASSETS).is_some_and(|rest| rest.starts_with('/'))
}

fn requests(manifest: &str) -> impl Iterator<Item = (&str, &str)> {
    manifest.lines().filter_map(|line| line.split_once('\t'))
}

pub fn unit_file(name: &str) -> Option<u32> {
    let number = name.strip_prefix(UNIT_FILE_PREFIX).and_then(|rest| rest.strip_suffix(UNIT_FILE_SUFFIX)).or_else(|| {
        name.strip_prefix(EXPLANATION_PREFIX)?.split_once(LANGUAGE_SEPARATOR).filter(|(_, language)| language.ends_with(UNIT_FILE_SUFFIX)).map(|(number, _)| number)
    })?;

    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    number.parse::<u32>().ok()?.checked_sub(1)
}

pub fn clean_name(name: &str) -> String {
    name.chars().filter(|c| !FORBIDDEN.contains(c)).collect()
}

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub save: Option<Save>,
    pub frames: Option<usize>,
    pub assets: usize,
    pub bytes: u64,
}

fn frames_in(text: &str) -> Option<usize> {
    parse(text).ok().map(|frames| frames.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // A forgiven fault's reason can hold newlines and tabs; the fault file keeps one record per line.
    #[test]
    fn fault_records_survive_a_round_trip() {
        let reason = "main_draw:imgcut_get_sprite_cut:9\treached 16\npast its limit of 1 \\ end";
        let text = format!("12\t{}\t{}\n", escape("main_draw.rs:9"), escape(reason));

        assert_eq!(parse_faults(&text), vec![Forgiven { frame: 12, site: "main_draw.rs:9".to_owned(), reason: reason.to_owned() }]);
        assert!(parse_faults("garbage without tabs\n").is_empty());
    }

    // A tape written before the tutorial flags and the Legend Quest redirect existed must still open.
    #[test]
    fn an_older_save_still_parses() {
        let text = r#"{"setup":{"stage":{"map_id":24052,"stage":9,"layout":null,"crown":0,"map_name":"","stage_name":""},"lineup":[],"tech":[],"treasures":[],"cannon":0,"style":0,"foundation":0,"parts":{},"items":[],"speed_engaged":false,"altar":null,"cat_god":"Present"}}"#;
        let save: Save = serde_json::from_str(text).expect("older save");

        assert!(save.setup.tutorial.battle_cleared && save.setup.tutorial.shop_seen);
        assert_eq!(save.setup.stage.dungeon_stage, None);
    }

    // Only a unit's own stats and description count as unit files; a hybrid replay falls back to nothing else.
    #[test]
    fn unit_files_are_numbered_from_one() {
        assert_eq!(unit_file("unit001.csv"), Some(0));
        assert_eq!(unit_file("Unit_Explanation45_en.csv"), Some(44));
        assert_eq!(unit_file("unit000.csv"), None);
        assert_eq!(unit_file("unitbuy.csv"), None);
        assert_eq!(unit_file("stage00.csv"), None);
    }
}
