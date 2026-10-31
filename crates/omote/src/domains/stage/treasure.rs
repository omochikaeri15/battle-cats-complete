use dioxus::prelude::*;

use codex::chapter::stage::{DropOdds, RewardStructure};
use kokoro::domains::stage::treasure;
use kokoro::domains::stage::{Stage, StageDataState};

use crate::systems::context::Shared;
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::Tip;

const MAX_ICON_SIZE: u32 = 32;

fn format_percent(value: f64) -> String {
    let rounded = format!("{value:.2}");

    format!("{}%", rounded.trim_end_matches('0').trim_end_matches('.'))
}

fn format_odds(odds: DropOdds) -> String {
    let first = format_percent(odds.first);

    if odds.repeat <= 0.0 {
        return first;
    }

    let repeat = format_percent(odds.repeat);

    if first == repeat { first } else { format!("{first} -> {repeat}") }
}

fn format_rule(drop_rule: i32) -> &'static str {
    match drop_rule {
        1 | 2 => "First Drop Once",
        -2..=0 => "Unlimited",
        -3 => "Guaranteed (Once)",
        -4 => "Guaranteed (Unlimited)",
        _ => "Unknown Rule",
    }
}

struct Row {
    left: String,
    item_id: u32,
    amount: u32,
}

pub(crate) fn view(stage: &Stage, data: &StageDataState, globals: &Shared) -> Element {
    match &stage.rewards {
        RewardStructure::Treasure { drop_rule, .. } => drop_table(&stage.rewards, format_rule(*drop_rule), data, globals),
        RewardStructure::Timed { scores, .. } => {
            let drops = drop_table(&stage.rewards, format_rule(-2), data, globals);
            let rows: Vec<Row> = scores.iter().map(|score| Row { left: score.score.to_string(), item_id: score.item_id, amount: score.amount }).collect();

            rsx! {
                div { class: "treasure-stack",
                    {drops}
                    if !rows.is_empty() {
                        {table("Timed Score Rewards".to_owned(), "Score", &rows, data, globals)}
                    }
                }
            }
        }
        RewardStructure::None => rsx! {},
    }
}

fn drop_table(rewards: &RewardStructure, rule: &str, data: &StageDataState, globals: &Shared) -> Element {
    let rows: Vec<Row> = rewards
        .drops()
        .iter()
        .zip(rewards.odds())
        .filter(|(_, odds)| odds.first > 0.0 || odds.repeat > 0.0)
        .map(|(drop, odds)| Row { left: format_odds(odds), item_id: drop.item_id, amount: drop.amount })
        .collect();

    if rows.is_empty() {
        return rsx! {};
    }

    table(format!("Treasure | {rule}"), "Chance", &rows, data, globals)
}

fn table(title: String, first_column: &str, rows: &[Row], data: &StageDataState, globals: &Shared) -> Element {
    let pictures = images::pictures();
    let vault = globals.0.vault();
    let vfs = &vault.vfs;

    rsx! {
        div { class: "section drop-table treasure-table",
            div { class: "section-title", "{title}" }
            div { class: "rule" }
            div { class: "zebra-table",
                div { class: "zebra-header zebra-cells three",
                    div { class: "zebra-cell small", "{first_column}" }
                    div { class: "zebra-cell small", "Item" }
                    div { class: "zebra-cell small", "Amount" }
                }
                for (index, row) in rows.iter().enumerate() {
                    {
                        let resolved = treasure::resolve_drop(vfs, row.item_id, row.amount, &vault.vds.items, &data.drop_chara_registry, &data.unit_buy_registry);
                        let icon = resolved.image_path.as_ref().and_then(|path| pictures.boxed(path, MAX_ICON_SIZE));
                        let class = if index % 2 == 0 { "zebra-row-even zebra-cells three" } else { "zebra-row-odd zebra-cells three" };

                        rsx! {
                            div { class: "{class}",
                                div { class: "zebra-cell", "{row.left}" }
                                div { class: "zebra-cell",
                                    match icon {
                                        Some(url) => rsx! { Tip { text: resolved.name.clone(), Pic { url, class: "drop-icon", alt: resolved.name.clone() } } },
                                        None => rsx! { span { "{resolved.name}" } },
                                    }
                                }
                                div { class: "zebra-cell", "{resolved.amount_display}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
