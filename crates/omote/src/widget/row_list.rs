use dioxus::prelude::*;

pub(crate) const ROW_HEIGHT: f32 = 50.0;
pub(crate) const ROW_GAP: f32 = 4.0;
const BUFFER_ROWS: usize = 3;
const DEFAULT_VIEWPORT: f32 = 600.0;

#[component]
pub(crate) fn RowList(count: usize, row: Callback<usize, Element>, empty: String, #[props(default = ROW_HEIGHT)] height: f32, #[props(default)] tail: f32, #[props(default = BUFFER_ROWS)] buffer: usize) -> Element {
    let mut scroll = use_signal(|| 0.0f32);
    let mut viewport = use_signal(|| DEFAULT_VIEWPORT);
    let stride = height + ROW_GAP;
    let first = ((scroll() / stride).floor().max(0.0) as usize).saturating_sub(buffer).min(count);
    let shown = (viewport() / stride).ceil().max(1.0) as usize + buffer * 2;
    let last = first.saturating_add(shown).min(count);
    let before = first as f32 * stride;
    let after = count.saturating_sub(last) as f32 * stride + tail;

    rsx! {
        div {
            class: "roster-list scroll",
            onscroll: move |event| scroll.set(event.data().scroll_top() as f32),
            onresize: move |event| {
                if let Ok(size) = event.data().get_content_box_size() {
                    viewport.set(size.height as f32);
                }
            },
            if count == 0 {
                div { class: "none-found", "{empty}" }
            }
            div { class: "row-spacer", style: "height: {before}px;" }
            for index in first..last {
                {row.call(index)}
            }
            div { class: "row-spacer", style: "height: {after}px;" }
        }
    }
}
