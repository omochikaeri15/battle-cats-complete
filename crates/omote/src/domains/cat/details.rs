use dioxus::prelude::*;

use kokoro::common::io;
use kokoro::domains::cat::combo::{self, CatCombo, ComboMember};

use crate::systems::context::Shared;
use crate::systems::images::{self, Pictures};
use crate::systems::picture::Pic;
use crate::widget::{FitColumn, Tip};

use super::{SelectedCat, jump, state};

const MATERIAL_CANVAS: u32 = 128;
const XP_ICON_ID: i32 = 6;
const XP_ICON_HEIGHT: f32 = 32.0;
const EVOLVE_TEXT_LINES: usize = 3;
const DESCRIPTION_LINES: usize = 3;
const COMBO_NAME_SIZE: f32 = 18.0;
const COMBO_DESCRIPTION_SIZE: f32 = 14.0;
const COMBO_CHARAGROUP_SIZE: f32 = 11.0;
const COMBO_UNI_SIZE: u32 = 68;

#[component]
pub(crate) fn Details(cat: SelectedCat, globals: Shared) -> Element {
    let state = state();
    let pictures = images::pictures();
    let form = (state.form)();
    let cat = cat.0;
    let mut lines = cat
        .description
        .get(form)
        .and_then(Option::as_ref)
        .filter(|lines| lines.iter().any(|line| !line.trim().is_empty()))
        .cloned()
        .unwrap_or_else(|| vec!["No description available".to_owned()]);

    lines.resize(lines.len().max(DESCRIPTION_LINES), String::new());

    let description = lines.join("\n");
    let combos = combo::combos(globals.0.context(), cat.id, form);

    rsx! {
        div { class: "details",
            div { class: "details-section",
                div { class: "details-heading", "Description" }
                pre { class: "details-text", "{description}" }
            }
            {evolve(&cat, form, &globals, &pictures)}
            if !combos.is_empty() {
                div { class: "details-section",
                    div { class: "details-heading", "Cat Combo" }
                    div { class: "combo-cards",
                        for held in combos.iter() {
                            {combo_card(held, cat.id, &pictures)}
                        }
                    }
                }
            }
        }
    }
}

fn evolve(cat: &kokoro::domains::cat::scanner::CatEntry, form: usize, globals: &Shared, pictures: &Pictures) -> Element {
    let vault = globals.0.vault();
    let vfs = &vault.vfs;
    let (materials, xp_cost) = match form {
        2 => (cat.unitbuy.true_form_materials().collect::<Vec<_>>(), cat.unitbuy.true_form_xp_cost),
        3 => (cat.unitbuy.ultra_form_materials().collect::<Vec<_>>(), cat.unitbuy.ultra_form_xp_cost),
        _ => return rsx! {},
    };
    let text = cat.evolve_text.texts.get(form).and_then(Option::as_ref).filter(|lines| !lines.is_empty()).map(|lines| {
        let mut padded = lines.clone();

        padded.resize(padded.len().max(EVOLVE_TEXT_LINES), String::new());
        padded.join("\n")
    });

    if materials.is_empty() && text.is_none() && xp_cost <= 0 {
        return rsx! {};
    }

    let xp = (xp_cost > 0).then(|| io::gatya_item_icon(vfs, XP_ICON_ID).and_then(|path| pictures.trimmed(&path))).flatten();

    rsx! {
        div { class: "details-section",
            div { class: "details-heading", "Evolve" }
            if let Some(text) = text {
                pre { class: "details-text", "{text}" }
            }
            if !materials.is_empty() {
                div { class: "materials",
                    for material in materials.iter() {
                        {
                            let icon = io::gatya_item_icon(vfs, material.item_id).and_then(|path| pictures.boxed(&path, MATERIAL_CANVAS));
                            let name = u32::try_from(material.item_id).ok().and_then(|id| vault.vds.items.name(vfs, id));
                            let item_id = material.item_id;
                            let quantity = material.quantity;
                            let inner = rsx! {
                                div { class: "material",
                                    match icon {
                                        Some(url) => rsx! { Pic { url } },
                                        None => rsx! { div { class: "material-placeholder", "ID {item_id}" } },
                                    }
                                    span { class: "badge", "×{quantity}" }
                                }
                            };

                            match name {
                                Some(name) => rsx! { Tip { text: name, {inner} } },
                                None => inner,
                            }
                        }
                    }
                }
            }
            if xp_cost > 0 {
                div { class: "xp-row",
                    match xp {
                        Some(sized) => {
                            let width = if sized.height > 0 { XP_ICON_HEIGHT * sized.width as f32 / sized.height as f32 } else { XP_ICON_HEIGHT };
                            rsx! { Pic { url: sized.url, style: format!("width: {width}px; height: {XP_ICON_HEIGHT}px;"), alt: "XP" } }
                        }
                        None => rsx! { div { class: "material-placeholder xp-placeholder", "XP" } },
                    }
                    span { class: "xp-cost bold", "{xp_cost}" }
                }
            }
        }
    }
}

fn combo_card(held: &CatCombo, cat_id: u32, pictures: &Pictures) -> Element {
    let mut lines = vec![(held.name.clone(), COMBO_NAME_SIZE)];

    if !held.effect.trim().is_empty() {
        lines.push((held.effect.clone(), COMBO_DESCRIPTION_SIZE));
    }

    if let Some(restriction) = &held.restriction {
        lines.push((restriction.clone(), COMBO_CHARAGROUP_SIZE));
    }

    rsx! {
        div { class: "card-outlined combo-card",
            div { class: "combo-text",
                FitColumn { lines }
            }
            div { class: "combo-slots",
                for member in held.members.iter() {
                    {member_slot(member, cat_id, pictures)}
                }
            }
        }
    }
}

fn member_slot(member: &ComboMember, cat_id: u32, pictures: &Pictures) -> Element {
    let navigator = use_navigator();
    let state = state();
    let icon = member.icon.as_ref().and_then(|source| pictures.boxed(&source.path, COMBO_UNI_SIZE));
    let label = member.label();
    let target = member.id.filter(|id| *id != cat_id).map(|id| (id, member.form));
    let hint = member.name.clone().or_else(|| label.clone());
    let slot = rsx! {
        div { class: "combo-slot",
            match icon {
                Some(url) => rsx! { Pic { url } },
                None => rsx! { div { class: "combo-slot-empty" } },
            }
            if let Some(label) = label {
                span { class: "badge centered", "{label}" }
            }
        }
    };
    let slot = match target {
        Some((id, form)) => rsx! { button { class: "combo-jump", onclick: move |_| jump(&navigator, id, form, state), {slot} } },
        None => slot,
    };

    match hint {
        Some(hint) => rsx! { Tip { text: hint, {slot} } },
        None => slot,
    }
}
