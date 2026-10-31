use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

const MAX_FONT_SIZE: f32 = 22.0;
const MIN_FONT_SIZE: f32 = 8.0;
const FONT_STEP: f32 = 0.5;
const LINE_HEIGHT: f32 = 1.3;
const MAX_LINES: f32 = 2.0;
const UNBROKEN_LINES: f32 = 1.0;
const SLACK: f32 = 0.5;

fn candidates() -> impl Iterator<Item = f32> {
    let steps = ((MAX_FONT_SIZE - MIN_FONT_SIZE) / FONT_STEP) as usize;

    (0..=steps).map(|step| MAX_FONT_SIZE - step as f32 * FONT_STEP)
}

fn allowed_lines(name: &str) -> f32 {
    if name.contains(char::is_whitespace) { MAX_LINES } else { UNBROKEN_LINES }
}

fn fits(size: f32, width: f32, height: f32, allowed: f32, wrap_width: f32) -> bool {
    let lines = (height / (size * LINE_HEIGHT)).round();

    lines <= allowed && width <= wrap_width + SLACK
}

#[component]
pub(crate) fn NameBox(name: String, width: f32, height: f32, wrap_width: f32) -> Element {
    let mut size: Signal<Option<f32>> = use_signal(|| None);
    let mut seen = use_signal(String::new);
    let probes: Rc<RefCell<Vec<(f32, MountedEvent)>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let mut probed = use_signal(|| 0usize);

    if *seen.peek() != name {
        seen.set(name.clone());
        size.set(None);
        probes.borrow_mut().clear();
    }

    let total = candidates().count();
    let gathered = Rc::clone(&probes);

    use_effect(move || {
        if probed() < total || size.peek().is_some() {
            return;
        }

        let allowed = allowed_lines(&seen.peek());
        let held: Vec<(f32, MountedEvent)> = gathered.borrow().clone();

        spawn(async move {
            let mut chosen = MIN_FONT_SIZE;

            for (candidate, probe) in held {
                if let Ok(rect) = probe.get_client_rect().await
                    && fits(candidate, rect.size.width as f32, rect.size.height as f32, allowed, wrap_width)
                {
                    chosen = candidate;
                    break;
                }
            }

            size.set(Some(chosen));
        });
    });

    let settled = size();
    let shown = settled.unwrap_or(MIN_FONT_SIZE);
    let visibility = if settled.is_some() { "visible" } else { "hidden" };

    rsx! {
        div { class: "name-box", style: "width: {width}px; height: {height}px;",
            span { class: "name-text", style: "max-width: {wrap_width}px; font-size: {shown}px; visibility: {visibility};", "{name}" }
            if settled.is_none() {
                for candidate in candidates() {
                    span {
                        key: "{candidate}",
                        class: "name-probe",
                        style: "max-width: {wrap_width}px; font-size: {candidate}px;",
                        onmounted: {
                            let probes = Rc::clone(&probes);

                            move |event: MountedEvent| {
                                probes.borrow_mut().push((candidate, event));
                                probed.set(probes.borrow().len());
                            }
                        },
                        "{name}"
                    }
                }
            }
        }
    }
}
