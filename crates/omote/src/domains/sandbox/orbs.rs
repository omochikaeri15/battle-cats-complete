use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;

use codex::cat::unit::Equipment;
use codex::combat::Identity;
use kokoro::Vfs;
use kokoro::domains::sandbox::orb::{self, Allowance, GRADES, Kind};

use crate::systems::context::Shared;
use crate::systems::images::{self, Pictures};
use crate::systems::picture::Pic;

const EXPLANATION: &str = "equipment_explonation.tsv";
const PLACEHOLDER: &str = "%@";
const UNIVERSAL: i32 = 12;
const BEST_GRADE: usize = 4;
const ATTRIBUTE_SHEET: &str = "equipment_attribute";
const EFFECT_SHEET: &str = "equipment_effect";
const GRADE_SHEET: &str = "equipment_grade";
pub(crate) const PREVIEW_SIZE: f32 = 56.0;
const NONE: &str = "None";

const TRAITS: [Identity; 12] = [
    Identity::TraitRed,
    Identity::TraitFloating,
    Identity::TraitDark,
    Identity::TraitMetal,
    Identity::TraitAngel,
    Identity::TraitAlien,
    Identity::TraitZombie,
    Identity::TraitRelic,
    Identity::TraitTraitless,
    Identity::TraitWitch,
    Identity::TraitEva,
    Identity::TraitAku,
];

pub(crate) struct Book {
    pub(crate) orbs: Arc<Vec<Equipment>>,
    kinds: Vec<Kind>,
    effects: Vec<String>,
    traits: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct Shelf(pub(crate) Rc<Book>);

impl PartialEq for Shelf {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn effect_names(vfs: &Vfs) -> Vec<String> {
    let named = vfs.variants(EXPLANATION).into_iter().next().unwrap_or_else(|| EXPLANATION.to_owned());

    vfs.find(named.as_str())
        .and_then(|path| vfs.read(&path).ok())
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
        .map_or_else(Vec::new, |content| {
            content
                .lines()
                .map(|line| {
                    let head = line.split('\t').next().unwrap_or_default();

                    head.split(PLACEHOLDER).next().unwrap_or_default().trim().trim_end_matches(':').trim().to_owned()
                })
                .collect()
        })
}

impl Book {
    pub(crate) fn load(globals: &Shared) -> Self {
        let vault = globals.0.vault();
        let orbs = vault.vds.cats.orbs(&vault.vfs);
        let kinds = orb::kinds(&orbs);
        let effects = effect_names(&vault.vfs);
        let traits = TRAITS.iter().map(|identity| globals.0.names.trait_label(*identity).to_owned()).collect();

        Self { orbs, kinds, effects, traits }
    }

    fn effect_label(&self, content: i32) -> String {
        usize::try_from(content)
            .ok()
            .and_then(|index| self.effects.get(index))
            .filter(|name| !name.is_empty())
            .map_or_else(|| format!("Effect {content}"), Clone::clone)
    }

    fn trait_label(&self, attribute: i32) -> String {
        usize::try_from(attribute).ok().and_then(|index| self.traits.get(index)).map_or_else(|| format!("Trait {attribute}"), Clone::clone)
    }

    fn aims(&self, content: i32, allowance: Allowance) -> Vec<i32> {
        orb::attributes(&self.kinds, content).into_iter().filter(|attribute| allowance.permits(Kind { content, attribute: Some(*attribute) })).collect()
    }

    fn offered(&self, allowance: Allowance) -> Vec<i32> {
        orb::effects(&self.kinds).into_iter().filter(|content| orb::attributes(&self.kinds, *content).is_empty() || !self.aims(*content, allowance).is_empty()).collect()
    }
}

pub(crate) fn picture(pictures: &Pictures, orbs: &[Equipment], orb: u32, size: f32) -> Element {
    let Some(held) = orbs.get(orb as usize) else {
        return rsx! { div { class: "orb-stack", style: "width: {size}px; height: {size}px;" } };
    };
    let face = pictures.cut_sized(ATTRIBUTE_SHEET, usize::try_from(held.attribute.unwrap_or(UNIVERSAL)).unwrap_or(UNIVERSAL as usize));
    let layers = [(EFFECT_SHEET, held.content), (GRADE_SHEET, held.grade_id)];
    let scale = face.as_ref().map_or(1.0, |face| if face.width > 0 { size / face.width as f32 } else { 1.0 });

    rsx! {
        div { class: "orb-stack", style: "width: {size}px; height: {size}px;",
            if let Some(face) = face.as_ref() {
                Pic { url: Rc::clone(&face.url), class: "orb-face" }
            }
            for (sheet, index) in layers {
                if let Some(over) = index.and_then(|index| usize::try_from(index).ok()).and_then(|index| pictures.cut_sized(sheet, index)) {
                    Pic {
                        url: over.url,
                        class: "orb-layer",
                        style: format!("width: {}px; height: {}px;", over.width as f32 * scale, over.height as f32 * scale),
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn Picker(shelf: Shelf, held: Option<u32>, allowance: Allowance, onpick: EventHandler<Option<u32>>) -> Element {
    let pictures = images::pictures();
    let book = &shelf.0;
    let kind = held.and_then(|orb| orb::kind(&book.orbs, orb));
    let offered = book.offered(allowance);
    let chosen = kind.map(|kind| kind.content).filter(|content| offered.contains(content));
    let aims = kind.map_or_else(Vec::new, |kind| book.aims(kind.content, allowance));
    let aim = kind.and_then(|kind| kind.attribute);
    let grades: Vec<usize> = kind.map_or_else(Vec::new, |kind| {
        let mut offered: Vec<usize> = orb::grades(&book.orbs, kind).into_iter().map(|(grade, _)| grade).collect();

        offered.sort_unstable();
        offered.dedup();
        offered
    });
    let grade = held.and_then(|orb| orb::grade(&book.orbs, orb));
    let pick_effect = {
        let shelf = shelf.clone();

        move |content: i32| {
            let book = &shelf.0;
            let grade = held.and_then(|orb| orb::grade(&book.orbs, orb)).unwrap_or(BEST_GRADE);
            let aimed = held.and_then(|orb| orb::kind(&book.orbs, orb)).and_then(|kind| kind.attribute);
            let offered = book.aims(content, allowance);
            let attribute = offered.iter().find(|attribute| Some(**attribute) == aimed).or_else(|| offered.first()).copied();

            onpick.call(orb::pick(&book.orbs, Kind { content, attribute }, grade));
        }
    };
    let pick_aim = {
        let shelf = shelf.clone();

        move |attribute: i32| {
            let book = &shelf.0;
            let Some(kind) = held.and_then(|orb| orb::kind(&book.orbs, orb)) else {
                return;
            };
            let grade = held.and_then(|orb| orb::grade(&book.orbs, orb)).unwrap_or(BEST_GRADE);

            onpick.call(orb::pick(&book.orbs, Kind { attribute: Some(attribute), ..kind }, grade));
        }
    };
    let pick_grade = {
        let shelf = shelf.clone();

        move |grade: usize| {
            let book = &shelf.0;
            let Some(kind) = held.and_then(|orb| orb::kind(&book.orbs, orb)) else {
                return;
            };

            onpick.call(orb::pick(&book.orbs, kind, grade));
        }
    };
    let effect_value = chosen.map_or_else(|| NONE.to_owned(), |content| book.effect_label(content));
    let aim_value = aim.map_or_else(|| NONE.to_owned(), |attribute| book.trait_label(attribute));
    let grade_value = grade.and_then(|grade| GRADES.get(grade)).map_or(NONE, |grade| *grade);

    rsx! {
        div { class: "orb-picker",
            match held {
                Some(orb) => picture(&pictures, &book.orbs, orb, PREVIEW_SIZE),
                None => rsx! { div { class: "orb-stack", style: "width: {PREVIEW_SIZE}px; height: {PREVIEW_SIZE}px;" } },
            }
            label { class: "orb-row",
                span { class: "orb-label", "Type" }
                select { class: "combo orb-pick", value: "{effect_value}",
                    onchange: {
                        let labels = Rc::clone(&shelf.0);

                        move |event| {
                            let picked = offered.iter().find(|content| labels.effect_label(**content) == event.value()).copied();

                            if let Some(content) = picked {
                                pick_effect(content);
                            }
                        }
                    },
                    option { value: "{NONE}", selected: chosen.is_none(), disabled: true, "{NONE}" }
                    for content in book.offered(allowance) {
                        option { value: "{book.effect_label(content)}", selected: chosen == Some(content), "{book.effect_label(content)}" }
                    }
                }
            }
            label { class: "orb-row",
                span { class: "orb-label", "Trait" }
                if aims.is_empty() {
                    div { class: "combo orb-pick stale", "{NONE}" }
                } else {
                    select { class: "combo orb-pick", value: "{aim_value}",
                        onchange: {
                            let book = Rc::clone(&shelf.0);
                            let aims = aims.clone();

                            move |event| {
                                if let Some(attribute) = aims.iter().find(|attribute| book.trait_label(**attribute) == event.value()).copied() {
                                    pick_aim(attribute);
                                }
                            }
                        },
                        option { value: "{NONE}", selected: aim.is_none(), disabled: true, "{NONE}" }
                        for attribute in aims.iter().copied() {
                            option { value: "{book.trait_label(attribute)}", selected: aim == Some(attribute), "{book.trait_label(attribute)}" }
                        }
                    }
                }
            }
            label { class: "orb-row",
                span { class: "orb-label", "Grade" }
                select { class: "combo orb-pick", value: "{grade_value}",
                    onchange: {
                        let grades = grades.clone();

                        move |event| {
                            if let Some(picked) = grades.iter().find(|grade| GRADES.get(**grade).copied() == Some(event.value().as_str())).copied() {
                                pick_grade(picked);
                            }
                        }
                    },
                    option { value: "{NONE}", selected: grade.is_none(), disabled: true, "{NONE}" }
                    for held_grade in grades.iter().copied() {
                        option { value: "{GRADES.get(held_grade).copied().unwrap_or(NONE)}", selected: grade == Some(held_grade), "{GRADES.get(held_grade).copied().unwrap_or(NONE)}" }
                    }
                }
            }
            button { class: "btn btn-danger btn-popup-action", disabled: held.is_none(), onclick: move |_| onpick.call(None), "{NONE}" }
        }
    }
}
