use eframe::egui;
use std::path::{Path, PathBuf};
use std::cell::RefCell;

use core::cat::logic::scanner::CatEntry;
use crate::global::sheet::GuiSpriteSheet;
use core::global::formats::mamodel::Model;
use crate::features::animation::viewer::AnimViewer;
use core::settings::logic::Settings;
use core::cat::paths::{self, AnimType};
use core::animation::logic::constants::{
    IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE
};
use crate::global::shared::DragGuard;

thread_local! {
    static PATH_CACHE: RefCell<(String, String, Vec<(usize, PathBuf)>, Option<(PathBuf, PathBuf, PathBuf)>, Option<(PathBuf, PathBuf, PathBuf, PathBuf)>)> = Default::default();
}

pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    cat_entry: &CatEntry,
    current_form: usize,
    anim_viewer: &mut AnimViewer,
    model_data: &mut Option<Model>,
    anim_sheet: &mut GuiSpriteSheet,
    settings: &mut Settings,
    drag_guard: &mut DragGuard,
) {
    let form_char = match current_form { 0 => 'f', 1 => 'c', 2 => 's', _ => 'u' };
    let primary_id = format!("{:03}_{}_{}", cat_entry.id, form_char, anim_viewer.texture_version);

    PATH_CACHE.with(|cache| {
        let mut c = cache.borrow_mut();

        if c.0 != primary_id {
            let root = Path::new(paths::DIR_CATS);
            let egg_ids = cat_entry.egg_ids;
            let priority = &settings.general.language_priority;

            let mut available_anims = Vec::new();
            let anim_defs = [IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE];
            for idx in anim_defs {
                let p = paths::maanim(root, cat_entry.id, current_form, egg_ids, idx);
                let parent = p.parent().unwrap();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap();

                if let Some(resolved) = core::global::get(parent, &[name], priority).into_iter().next() {
                    available_anims.push((idx, resolved));
                }
            }

            let resolve = |p: PathBuf| {
                let parent = p.parent()?;
                let name = p.file_name()?.to_str()?;
                core::global::get(parent, &[name], priority).into_iter().next()
            };

            let primary_assets = (|| {
                let png = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Png))?;
                let cut = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Imgcut))?;
                let model = resolve(paths::anim(root, cat_entry.id, current_form, egg_ids, AnimType::Mamodel))?;
                Some((png, cut, model))
            })();

            let mut secondary_assets = None;
            let mut secondary_id = String::new();

            if let Some(Some(stats)) = cat_entry.stats.get(current_form) {
                if stats.conjure_unit_id > 0 {
                    let s_id = stats.conjure_unit_id as u32;
                    secondary_assets = (|| {
                        let png = resolve(paths::anim(root, s_id, 0, (-1, -1), AnimType::Png))?;
                        let cut = resolve(paths::anim(root, s_id, 0, (-1, -1), AnimType::Imgcut))?;
                        let model = resolve(paths::anim(root, s_id, 0, (-1, -1), AnimType::Mamodel))?;
                        let atk = resolve(paths::maanim(root, s_id, 0, (-1, -1), 2))?;
                        Some((png, cut, model, atk))
                    })();
                    if secondary_assets.is_some() {
                        secondary_id = format!("spirit_{}_{}", s_id, anim_viewer.texture_version);
                    }
                }
            }

            // Update Cache
            c.0 = primary_id.clone();
            c.1 = secondary_id;
            c.2 = available_anims;
            c.3 = primary_assets;
            c.4 = secondary_assets;
        }

        // Pass cached data straight to the viewer memory
        anim_viewer.show(ui, ctx, &c.0, &c.1, &c.2, c.3.clone(), c.4.clone(), model_data, anim_sheet, settings, drag_guard);
    });
}