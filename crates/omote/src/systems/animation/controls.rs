use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::systems::animation::Placement;

use crate::platform::{Platform, Surface};
use crate::systems::export::{self as feedback, Phase, class};

use super::{COLUMNS, HOLD_DELAY_MS, Held, Motions, ViewerState, clamp_frame, export, offset_choices, offset_of, selected_index, state, step};

const DISPLAY_CEILING: i32 = 999_999;
const ANIM_TEXT_STEPS: [f32; 6] = [13.0, 12.0, 11.0, 10.0, 9.0, 8.0];
const ANIM_TEXT_FLOOR: f32 = 8.0;
const ANIM_BUTTON_W: f32 = 70.0;
const ANIM_BUTTON_H: f32 = 25.0;
const ANIM_LABEL_INSET: f32 = 2.0;
const GLYPH_ADVANCE: f32 = 0.52;
const LINE_HEIGHT: f32 = 1.1;
const GRID_ROWS: usize = 2;
const PLAY_GLYPH: &str = "\u{25B6}";
const PAUSE_GLYPH: &str = "\u{25AE}\u{25AE}";

fn fitted_size(label: &str) -> f32 {
    let room_w = ANIM_BUTTON_W - ANIM_LABEL_INSET * 2.0;
    let room_h = ANIM_BUTTON_H - ANIM_LABEL_INSET * 2.0;
    let glyphs = label.chars().count();

    ANIM_TEXT_STEPS
        .into_iter()
        .find(|size| {
            let per_line = (room_w / (size * GLYPH_ADVANCE)).floor().max(1.0);
            let lines = (room_h / (size * LINE_HEIGHT)).floor().max(1.0);

            glyphs as f32 <= per_line * lines
        })
        .unwrap_or(ANIM_TEXT_FLOOR)
}

fn display_bound(bound: Option<i32>) -> String {
    match bound {
        Some(value) if value <= DISPLAY_CEILING => value.to_string(),
        _ => "???".to_owned(),
    }
}

fn matches(choice: Placement, current: Option<usize>) -> bool {
    match (choice, current) {
        (Placement::Bare, None) => true,
        (Placement::Row(row), Some(current)) => row == current,
        _ => false,
    }
}

fn release(mut state: ViewerState) {
    let Some((direction, held_ms)) = *state.hold.peek() else {
        return;
    };

    state.hold.set(None);

    if held_ms <= HOLD_DELAY_MS {
        step(state, f32::from(direction));
    }
}

fn set_range_start(mut state: ViewerState, value: String) {
    if value.is_empty() {
        state.loop_start.set(None);
    } else if let Ok(parsed) = value.parse::<f32>() {
        let start = parsed.max(0.0);

        state.loop_start.set(Some(start));

        if *state.frame.peek() < start {
            state.frame.set(start);
        }
    }

    state.range_inputs.with_mut(|inputs| inputs.0 = value);
}

fn set_range_end(mut state: ViewerState, value: String) {
    if value.is_empty() {
        state.loop_end.set(None);
    } else if let Ok(parsed) = value.parse::<f32>() {
        let end = parsed.max(0.0);

        state.loop_end.set(Some(end));

        if *state.frame.peek() > end {
            state.frame.set(end);
        }
    }

    state.range_inputs.with_mut(|inputs| inputs.1 = value);
}

fn set_frame(mut state: ViewerState, value: &str) {
    let trimmed = value.trim();
    let target = if trimmed.is_empty() { Some(0.0) } else { trimmed.parse::<f32>().ok() };

    if let Some(target) = target {
        state.frame.set(target);
        clamp_frame(state);
    }
}

#[component]
pub(super) fn Controls(motions: Memo<Motions>, loaded: Memo<Held>, slots: Memo<Vec<Option<usize>>>, surface: Signal<Option<Rc<dyn Surface>>>) -> Element {
    let mut state = state();
    let (motions_memo, loaded_memo, slots_memo) = (motions, loaded, slots);
    let motions = motions.read();
    let loaded = loaded.read();
    let slots = slots.read();
    let label = (state.selected_label)();
    let index = selected_index(&motions.set, &slots, label.as_deref());
    let motion = index.and_then(|index| motions.set.motions.get(index));
    let rig = loaded.0.rig.as_deref();
    let base_available = rig.is_some();
    let anim_loaded = base_available && motion.is_some();
    let is_model = motion.is_some_and(|motion| motion.file.is_none());
    let expanded = (state.controls_expanded)();
    let playing = (state.playing)();
    let frame_text = ((state.frame)().trunc() as i32).to_string();
    let display_max = display_bound((state.active)().as_ref().and_then(super::Active::loop_bound));
    let (range_start, range_end) = (state.range_inputs)();
    let effective_max = if is_model {
        "0".to_owned()
    } else {
        (state.loop_end)().map_or_else(|| display_max.clone(), |end| (end.trunc() as i32).to_string())
    };
    let choices = offset_choices(&motions.set, rig);
    let current_offset = offset_of(rig, (state.placement)());
    let current_label = choices.iter().find(|(_, choice)| matches(*choice, current_offset)).map_or_else(String::new, |(name, _)| name.clone());
    let picks = choices.clone();
    let tiles = slots.iter().enumerate().map(|(at, slot)| {
        let Some(held) = slot.and_then(|index| motions.set.motions.get(index)) else {
            return rsx! { div { key: "{at}", class: "motion-tile vacant" } };
        };
        let tile_label = held.label();
        let size = fitted_size(&tile_label);
        let class = if *slot == index { "motion-tile active" } else { "motion-tile" };
        let chosen = tile_label.clone();

        rsx! {
            button {
                key: "{at}",
                class: "{class}",
                style: "font-size: {size}px;",
                onclick: move |_| {
                    state.selected_label.set(Some(chosen.clone()));
                    state.frame.set(0.0);
                },
                "{tile_label}"
            }
        }
    });
    let grid_class = if slots.len() > COLUMNS * GRID_ROWS { "motion-grid scrolls" } else { "motion-grid" };
    let mut drawer_height = use_signal(|| 0.0f32);
    let measured = drawer_height();
    let drawer_style = match (expanded, measured > 0.0) {
        (true, true) => format!("height: {measured}px;"),
        (true, false) => "height: auto;".to_owned(),
        (false, _) => "height: 0px;".to_owned(),
    };
    let platform = use_context::<Platform>();
    let mut phase = use_signal(|| Phase::Idle);
    let exporting = phase() == Phase::Busy;
    let export_label = feedback::label(phase(), if playing { "Export WebP" } else { "Export PNG" }, "Exporting...", "Exported!");
    let export_class = format!("{} viewer-export", class(phase()));
    let shooter = platform.clone();

    use_effect(move || {
        let Some(_) = (state.region)() else {
            return;
        };

        if *phase.peek() != Phase::Idle {
            return;
        }

        let (motions, loaded, slots) = (motions_memo.peek().clone(), loaded_memo.peek().clone(), slots_memo.peek().clone());

        phase.set(Phase::Busy);
        spawn(export::run(shooter.clone(), state, surface, motions, loaded, slots, phase));
    });

    rsx! {
        div { class: "anim-controls",
            button { class: "btn btn-text anim-toggle", onclick: move |_| state.controls_expanded.toggle(), if expanded { "▼" } else { "▲" } }
            div { class: "anim-drawer", style: "{drawer_style}",
                div {
                    class: "anim-drawer-inner",
                    onresize: move |event| {
                        if let Ok(size) = event.data().get_border_box_size() {
                            drawer_height.set(size.height as f32);
                        }
                    },
                div { class: "rule" }
                div { class: "{grid_class}", {tiles} }
                div { class: "rule" }
                div { class: "transport-row",
                    div { class: "transport-col",
                        button {
                            class: "btn btn-toggle control-btn symbols",
                            disabled: !anim_loaded,
                            onclick: move |_| state.playing.toggle(),
                            if playing { "{PAUSE_GLYPH}" } else { "{PLAY_GLYPH}" }
                        }
                        button { class: "btn btn-toggle control-btn", disabled: !base_available, onclick: move |_| state.pan.set((0.0, 0.0)), "Orient" }
                    }
                    div { class: "transport-divider" }
                    div { class: "transport-col",
                        if playing {
                            div { class: "tile-row",
                                input {
                                    class: "tile tile-input range",
                                    r#type: "text",
                                    inputmode: "numeric",
                                    placeholder: "0",
                                    disabled: !anim_loaded,
                                    value: "{range_start}",
                                    oninput: move |event| set_range_start(state, event.value()),
                                }
                                div { class: "tile tilde", "~" }
                                input {
                                    class: "tile tile-input range",
                                    r#type: "text",
                                    inputmode: "numeric",
                                    placeholder: "{display_max}",
                                    disabled: !anim_loaded,
                                    value: "{range_end}",
                                    oninput: move |event| set_range_end(state, event.value()),
                                }
                            }
                        } else {
                            div { class: "tile-row",
                                button {
                                    class: "step-btn symbols",
                                    disabled: !anim_loaded,
                                    onpointerdown: move |_| state.hold.set(Some((-1, 0.0))),
                                    onpointerup: move |_| release(state),
                                    onpointerleave: move |_| state.hold.set(None),
                                    onpointercancel: move |_| state.hold.set(None),
                                    "◀"
                                }
                                input {
                                    class: "tile tile-input frame",
                                    r#type: "text",
                                    inputmode: "numeric",
                                    placeholder: "0",
                                    disabled: !anim_loaded,
                                    value: "{frame_text}",
                                    oninput: move |event| set_frame(state, &event.value()),
                                }
                                button {
                                    class: "step-btn symbols",
                                    disabled: !anim_loaded,
                                    onpointerdown: move |_| state.hold.set(Some((1, 0.0))),
                                    onpointerup: move |_| release(state),
                                    onpointerleave: move |_| state.hold.set(None),
                                    onpointercancel: move |_| state.hold.set(None),
                                    "▶"
                                }
                            }
                        }
                        div { class: "tile-row",
                            div { class: "tile range", "{frame_text}" }
                            div { class: "tile tilde", "/" }
                            div { class: "tile range", "{effective_max}" }
                        }
                    }
                    div { class: "transport-divider" }
                    div { class: "transport-col",
                        button {
                            class: "{export_class}",
                            disabled: !base_available || exporting,
                            onclick: move |_| {
                                if exporting {
                                    return;
                                }

                                state.region.set(None);
                                state.selection.set(None);
                                state.selecting.set(true);
                            },
                            "{export_label}"
                        }
                        select {
                            class: "combo offset-select",
                            disabled: !base_available,
                            value: "{current_label}",
                            onchange: move |event| {
                                let picked = event.value();

                                if let Some((_, placement)) = picks.iter().find(|(name, _)| *name == picked) {
                                    state.placement.set(*placement);
                                    clamp_frame(state);
                                }
                            },
                            for (name, _) in choices {
                                option { key: "{name}", value: "{name}", selected: name == current_label, "{name}" }
                            }
                        }
                    }
                }
                }
            }
        }
    }
}
