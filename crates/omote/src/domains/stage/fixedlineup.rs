use dioxus::prelude::*;

use codex::chapter::stage::{AbilityType, CannonType, CertificationPreset, EvolutionForm, TreasureType};
use kokoro::domains::cat::waiter::unitexplanation;
use kokoro::domains::stage::fixedlineup::{ResolvedFixedLineup, ResolvedSlot};

use crate::app::Route;
use crate::domains::cat;
use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::Tip;

const ABILITIES: [(AbilityType, &str); 10] = [
    (AbilityType::CatCannonAttack, "Cat Cannon Attack"),
    (AbilityType::CatCannonRange, "Cat Cannon Range"),
    (AbilityType::CatCannonCharge, "Cat Cannon Charge"),
    (AbilityType::WorkerCatRate, "Worker Cat Efficiency"),
    (AbilityType::WorkerCatWallet, "Worker Cat Wallet"),
    (AbilityType::BaseDefense, "Cat Base Health"),
    (AbilityType::Research, "Research"),
    (AbilityType::BountyUp, "Accounting"),
    (AbilityType::Study, "Study"),
    (AbilityType::CatEnergy, "Cat Energy"),
];

const TREASURES: [(TreasureType, &str); 9] = [
    (TreasureType::EoC1, "EoC Ch. 1"),
    (TreasureType::EoC2, "EoC Ch. 2"),
    (TreasureType::EoC3, "EoC Ch. 3"),
    (TreasureType::ItF1, "ItF Ch. 1"),
    (TreasureType::ItF2, "ItF Ch. 2"),
    (TreasureType::ItF3, "ItF Ch. 3"),
    (TreasureType::CotC1, "CotC Ch. 1"),
    (TreasureType::CotC2, "CotC Ch. 2"),
    (TreasureType::CotC3, "CotC Ch. 3"),
];

pub(crate) fn view(lineup: &ResolvedFixedLineup, preset: &CertificationPreset, globals: &Shared) -> Element {
    rsx! {
        div { class: "section lineup-section",
            div { class: "section-title", "Fixed Lineup" }
            div { class: "rule" }
            div { class: "lineup-body",
                div { class: "lineup-slots",
                    div { class: "lineup-row",
                        for slot in lineup.slots.iter().take(5) {
                            {slot_view(slot, preset, globals)}
                        }
                    }
                    div { class: "lineup-row",
                        for slot in lineup.slots.iter().skip(5).take(5) {
                            {slot_view(slot, preset, globals)}
                        }
                    }
                }
                div { class: "card-outlined upgrades-panel scroll",
                    {upgrades(preset, globals)}
                }
            }
        }
    }
}

fn slot_view(slot: &ResolvedSlot, preset: &CertificationPreset, globals: &Shared) -> Element {
    let pictures = images::pictures();
    let navigator = use_navigator();
    let cat_state = cat::state();
    let vfs = &globals.0.vault().vfs;
    let Some(sized) = slot.image_path.as_ref().and_then(|path| pictures.trimmed(path)) else {
        return rsx! { div { class: "lineup-slot empty" } };
    };
    let image = rsx! { Pic { url: sized.url, class: "lineup-icon" } };
    let (Some(unit_id), Some(level)) = (slot.unit_id, slot.level) else {
        return rsx! { div { class: "lineup-slot", {image} } };
    };
    let explanation = unitexplanation(vfs, unit_id);
    let form = preset.characters.get(&unit_id).map_or(0, |chara| match chara.evolution_form {
        EvolutionForm::Normal => 0,
        EvolutionForm::Evolved => 1,
        EvolutionForm::True => 2,
        EvolutionForm::Ultra => 3,
        _ => 0,
    });
    let name = explanation.names.get(form).or_else(|| explanation.names.first()).and_then(|name| name.clone()).unwrap_or_else(|| format!("Unit {unit_id:03}"));
    let plus = slot.plus_level.unwrap_or(0);
    let plus_text = if plus > 0 { format!("+{plus}") } else { String::new() };
    let tip = format!("[Name] {name}\n[ID] {unit_id:03}\n[Level] {level}{plus_text}");

    rsx! {
        Tip { text: tip,
            button {
                class: "lineup-slot",
                onclick: move |_| {
                    let mut cat_state = cat_state;

                    cat_state.seeded_for.set(Some(unit_id));
                    cat_state.form.set(form);
                    cat_state.set_level_input(if plus > 0 { format!("{level}+{plus}") } else { level.to_string() });
                    navigator.push(Route::cat_form(unit_id, form));
                },
                {image}
            }
        }
    }
}

fn upgrades(preset: &CertificationPreset, globals: &Shared) -> Element {
    let (cannon_id, fallback) = match preset.slot_cannon_type {
        CannonType::Basic => (0, "Basic"),
        CannonType::SlowBeam => (1, "Slow Beam"),
        CannonType::IronWall => (2, "Iron Wall"),
        CannonType::Thunderbolt => (3, "Thunderbolt"),
        CannonType::Waterblast => (4, "Waterblast"),
        CannonType::HolyBlast => (5, "Holy Blast"),
        CannonType::Breakerblast => (6, "Breakerblast"),
        CannonType::Curseblast => (7, "Curseblast"),
        CannonType::Unknown(id) => (i32::from(id), "Unknown"),
    };
    let cannon_name = globals.0.names.cannon(cannon_id).unwrap_or(fallback).to_owned();
    let cannon_level = preset.cannon_levels.get(&preset.slot_cannon_type).copied().unwrap_or(0);
    let abilities: Vec<(String, String)> = ABILITIES
        .iter()
        .map(|(kind, name)| {
            let level = preset.abilities.get(kind).map_or_else(|| "0".to_owned(), |ability| {
                if ability.plus_level > 0 { format!("{} +{}", ability.level, ability.plus_level) } else { ability.level.to_string() }
            });

            ((*name).to_owned(), level)
        })
        .collect();
    let treasures: Vec<(String, String)> = TREASURES
        .iter()
        .map(|(kind, name)| {
            let grades = preset.treasures.get(kind).map_or_else(|| "0/0/0".to_owned(), |treasure| {
                format!("{}/{}/{}", treasure.inferior_count, treasure.normal_count, treasure.superior_count)
            });

            ((*name).to_owned(), grades)
        })
        .collect();

    rsx! {
        div { class: "upgrades",
            {zebra("Cannon", "Level", &[(cannon_name, cannon_level.to_string())])}
            div { class: "upgrade-pair",
                {zebra("Upgrade", "Level", &abilities)}
                {zebra("Chapter", "Treasure", &treasures)}
            }
        }
    }
}

fn zebra(label: &str, value: &str, rows: &[(String, String)]) -> Element {
    rsx! {
        div { class: "zebra-table tiny-table",
            div { class: "zebra-header zebra-cells two",
                div { class: "zebra-cell bold", "{label}" }
                div { class: "zebra-cell bold", "{value}" }
            }
            for (index, (left, right)) in rows.iter().enumerate() {
                div { class: if index % 2 == 0 { "zebra-row-even zebra-cells two" } else { "zebra-row-odd zebra-cells two" },
                    div { class: "zebra-cell", "{left}" }
                    div { class: "zebra-cell", "{right}" }
                }
            }
        }
    }
}
