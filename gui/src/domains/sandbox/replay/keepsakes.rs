use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use iced::Size;
use nyanko::files::{Localizable, Param};

use kore::common::context::GlobalContext;
use kore::common::formats::{fitting_cut, imgcut};
use kore::domains::sandbox::replay::{self as tape, Save, Summary};
use kore::domains::settings::{ScannerConfig, Settings};
use kore::{Source, Vault};

use crate::app::state::AppState;
use crate::common::sheet_layers;
use crate::domains::cat;

use super::{Message, State};

const SHEET_IMAGE: &str = ".png";
const WINDOW: Size = Size::new(1280.0, 720.0);

pub(crate) fn gather(vault: &Vault, save: Save, config: &ScannerConfig) -> Vec<(Box<str>, PathBuf)> {
    let staged = Arc::new(tape::staged(Summary { save: Some(save), ..Summary::default() }, vault.fork(), config));
    let (param, localizable) = (Param::default(), Localizable::default());
    let (mut settings, mut app_state) = (Settings::default(), AppState::default());
    let mut page = State::new();

    page.staged = Some(Arc::clone(&staged));
    drop(page.adopt(Arc::clone(&staged)));
    drop(page.view(0.0));

    for slot in 0..page.tiles.len() {
        let ctx = GlobalContext { param: &param, localizable: &localizable, vault: &staged.vault };

        page.open_unit(slot);
        drop(page.unit_popup_view(WINDOW, &settings, &app_state, ctx));

        let walk = page.tiles.get(slot).and_then(|tile| page.inspector.cat(tile.id)).map(cat::popup_walk).unwrap_or_default();

        for step in walk {
            drop(page.update(Message::Cat(step), &mut settings, &mut app_state, ctx));
            drop(page.unit_popup_view(WINDOW, &settings, &app_state, ctx));
        }
    }

    let traced = staged.vault.vfs.traced();
    let shadowed = shadowed_layers(&staged.vault, &traced);

    traced
        .into_iter()
        .filter(|path| !shadowed.contains(path))
        .filter_map(|path| Some((Box::from(path.file_name()?.to_str()?), path)))
        .collect()
}

fn drawable(png: &Source, cut: &Source) -> BTreeSet<usize> {
    let Some(sheet) = imgcut::parse(png, cut) else {
        return BTreeSet::new();
    };
    let Some(image) = sheet.image_data.as_ref() else {
        return BTreeSet::new();
    };

    sheet
        .cuts
        .iter()
        .enumerate()
        .filter(|(_, cut)| fitting_cut(cut, image.width(), image.height()).is_some())
        .map(|(icon, _)| icon)
        .collect()
}

fn layered_sheets(probe: &Vault, traced: &BTreeSet<PathBuf>) -> BTreeSet<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();

    for name in traced.iter().filter_map(|path| path.file_name()?.to_str()) {
        if let Some(base) = probe.vfs.stripped(name).filter(|base| base.ends_with(SHEET_IMAGE)) {
            *seen.entry(base).or_default() += 1;
        }
    }

    seen.into_iter().filter(|(_, layers)| *layers > 1).filter_map(|(base, _)| base.strip_suffix(SHEET_IMAGE).map(str::to_owned)).collect()
}

fn shadowed_layers(probe: &Vault, traced: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    let mut shadowed = BTreeSet::new();

    for sheet in layered_sheets(probe, traced) {
        let mut covered: BTreeSet<usize> = BTreeSet::new();

        for layer in sheet_layers(&probe.vfs, &sheet, false) {
            let Some(cut) = layer.imgcut.as_ref() else {
                continue;
            };
            let icons = drawable(&layer.png, cut);

            if icons.is_subset(&covered) {
                let own = cut.path.file_stem().and_then(|stem| stem.to_str()) == Some(layer.stem.as_str());

                if own {
                    shadowed.insert(cut.path.clone());
                }

                shadowed.insert(layer.png.path);
            } else {
                covered.extend(icons);
            }
        }
    }

    shadowed
}
