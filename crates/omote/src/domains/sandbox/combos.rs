use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::domains::cat::combo::{self, CatCombo};
use kokoro::domains::sandbox::Lineup;

use crate::platform::Platform;
use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::RowList;

use super::{Cats, Ruled};

const DOUBLE_CLICK_MS: f64 = 400.0;
const ROW_HEIGHT: f32 = 58.0;
const PLAY_CLEARANCE: f32 = 60.0;
const LOOKAHEAD_ROWS: usize = 12;
const EVERY_KIND: &str = "All Types";
const ANY_STRENGTH: &str = "Any Strength";
const ACTIVE: &str = "Active";
const INACTIVE: &str = "Inactive";
const BANNED_NOTE: &str = "Disabled on this stage";
const SLOT_COUNT: usize = 5;

#[derive(Clone)]
pub(crate) struct Held(pub(crate) Rc<Lineup>);

impl PartialEq for Held {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

struct Card {
    combo: CatCombo,
    folded: String,
}

#[derive(Clone, PartialEq, Default)]
enum Strength {
    #[default]
    Any,
    Active,
    Inactive,
    Band(i32),
}

struct Deck {
    cards: Vec<Card>,
    kinds: Vec<(i32, String)>,
    bands: Vec<(i32, String)>,
}

#[derive(Clone)]
struct Decked(Rc<Deck>);

impl PartialEq for Decked {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn build(globals: &Shared, cats: &Cats) -> Deck {
    let cards: Vec<Card> = combo::every(globals.0.context(), &cats.0).into_iter().map(|combo| Card { folded: combo.name.to_lowercase(), combo }).collect();
    let mut kinds: Vec<(i32, String)> = Vec::new();
    let mut bands: Vec<(i32, String)> = Vec::new();

    for card in &cards {
        if !kinds.iter().any(|(effect, _)| *effect == card.combo.effect_type) {
            kinds.push((card.combo.effect_type, card.combo.kind.clone()));
        }

        if !card.combo.band.is_empty() && !bands.iter().any(|(level, _)| *level == card.combo.effect_level) {
            bands.push((card.combo.effect_level, card.combo.band.clone()));
        }
    }

    kinds.sort_by_key(|(effect, _)| *effect);
    bands.sort_by_key(|(level, _)| *level);

    Deck { cards, kinds, bands }
}

#[component]
pub(crate) fn Combos(cats: Cats, globals: Shared, rules: Ruled, lineup: Held, focus: Option<(u32, usize)>, onadopt: EventHandler<usize>) -> Element {
    let platform = use_context::<Platform>();
    let pictures = images::pictures();
    let mut query = use_signal(String::new);
    let mut kind: Signal<Option<i32>> = use_signal(|| None);
    let mut strength = use_signal(Strength::default);
    let mut clicked: Signal<Option<(usize, f64)>> = use_signal(|| None);
    let shared = globals.clone();
    let deck = use_memo(use_reactive!(|(cats, shared)| Decked(Rc::new(build(&shared, &cats)))));
    let deck = deck();
    let rows = {
        let vault = globals.0.vault();

        vault.vds.cats.combos(&vault.vfs)
    };
    let wanted = query.read().to_lowercase();
    let strength_now = strength();
    let kind_now = kind();
    let mut shown: Vec<(usize, bool, bool)> = deck
        .0
        .cards
        .iter()
        .enumerate()
        .filter_map(|(index, card)| {
            let banned = rules.0.bans(card.combo.effect_type);
            let active = !banned && rows.get(card.combo.line).is_some_and(|row| lineup.0.active(row));
            let strong = match strength_now {
                Strength::Any => true,
                Strength::Active => active,
                Strength::Inactive => !active,
                Strength::Band(level) => card.combo.effect_level == level,
            };
            let joined = focus.is_none_or(|(unit, form)| card.combo.members.iter().any(|member| member.id == Some(unit) && member.form <= form));

            (joined && strong && kind_now.is_none_or(|effect| card.combo.effect_type == effect) && card.folded.contains(&wanted)).then_some((index, active, banned))
        })
        .collect();

    shown.sort_by(|left, right| {
        let (held_left, held_right) = (&deck.0.cards[left.0], &deck.0.cards[right.0]);

        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| held_left.combo.effect_type.cmp(&held_right.combo.effect_type))
            .then_with(|| held_right.combo.effect_level.cmp(&held_left.combo.effect_level))
            .then_with(|| held_left.combo.name.cmp(&held_right.combo.name))
    });

    let shown = Rc::new(shown);
    let count = shown.len();
    let listed = Rc::clone(&shown);
    let cards = deck.clone();
    let kind_label = kind_now.and_then(|effect| deck.0.kinds.iter().find(|(held, _)| *held == effect)).map_or(EVERY_KIND, |(_, label)| label.as_str()).to_owned();
    let strength_label = match &strength_now {
        Strength::Any => ANY_STRENGTH.to_owned(),
        Strength::Active => ACTIVE.to_owned(),
        Strength::Inactive => INACTIVE.to_owned(),
        Strength::Band(level) => deck.0.bands.iter().find(|(held, _)| held == level).map_or_else(|| ANY_STRENGTH.to_owned(), |(_, label)| label.clone()),
    };
    let kinds = deck.clone();
    let bands = deck.clone();
    let row = Callback::new(move |index: usize| {
        let Some((at, active, banned)) = listed.get(index).copied() else {
            return rsx! {};
        };
        let card = &cards.0.cards[at];
        let line = card.combo.line;
        let class = match (active, banned) {
            (_, true) => "combo-row banned",
            (true, false) => "combo-row active",
            (false, false) => "combo-row",
        };
        let note = match (&card.combo.restriction, banned) {
            (_, true) => Some(BANNED_NOTE.to_owned()),
            (Some(restriction), false) => Some(restriction.clone()),
            (None, false) => None,
        };
        let platform = platform.clone();

        rsx! {
            div {
                key: "{line}",
                class: "{class}",
                onclick: move |_| {
                    let now = platform.host().now();
                    let doubled = clicked.peek().is_some_and(|(held, at)| held == line && now - at <= DOUBLE_CLICK_MS);

                    clicked.set((!doubled).then_some((line, now)));

                    if doubled {
                        onadopt.call(line);
                    }
                },
                div { class: "combo-labels",
                    span { class: "bold combo-name", "{card.combo.name}" }
                    span { class: "combo-effect", "{card.combo.effect}" }
                    if let Some(note) = note {
                        span { class: "combo-note", "{note}" }
                    }
                }
                div { class: "combo-icons",
                    for (slot, member) in card.combo.members.iter().enumerate().take(SLOT_COUNT) {
                        {
                            let icon = member.icon.as_ref().and_then(|source| pictures.trimmed(&source.path));

                            rsx! {
                                div { key: "{slot}", class: "combo-icon",
                                    if let Some(icon) = icon {
                                        Pic { url: icon.url }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "card-outlined combos-card",
            div { class: "combos-header",
                span { class: "bold combos-title", "Combos" }
                input { class: "input combos-search", r#type: "search", placeholder: "Name...", value: "{query}", oninput: move |event| query.set(event.value()) }
                select { class: "combo combos-kind", value: "{kind_label}",
                    onchange: move |event| {
                        kind.set(kinds.0.kinds.iter().find(|(_, label)| *label == event.value()).map(|(effect, _)| *effect));
                    },
                    option { value: "{EVERY_KIND}", selected: kind_now.is_none(), "{EVERY_KIND}" }
                    for (effect, label) in deck.0.kinds.iter() {
                        option { value: "{label}", selected: kind_now == Some(*effect), "{label}" }
                    }
                }
                select { class: "combo combos-strength", value: "{strength_label}",
                    onchange: move |event| {
                        let value = event.value();
                        let picked = match value.as_str() {
                            ANY_STRENGTH => Strength::Any,
                            ACTIVE => Strength::Active,
                            INACTIVE => Strength::Inactive,
                            other => bands.0.bands.iter().find(|(_, label)| label == other).map_or(Strength::Any, |(level, _)| Strength::Band(*level)),
                        };

                        strength.set(picked);
                    },
                    option { value: "{ANY_STRENGTH}", selected: strength_now == Strength::Any, "{ANY_STRENGTH}" }
                    option { value: "{ACTIVE}", selected: strength_now == Strength::Active, "{ACTIVE}" }
                    option { value: "{INACTIVE}", selected: strength_now == Strength::Inactive, "{INACTIVE}" }
                    for (level, label) in deck.0.bands.iter() {
                        option { value: "{label}", selected: strength_now == Strength::Band(*level), "{label}" }
                    }
                }
            }
            RowList { count, row, empty: "No Combos Found!", height: ROW_HEIGHT, tail: PLAY_CLEARANCE, buffer: LOOKAHEAD_ROWS }
        }
    }
}
