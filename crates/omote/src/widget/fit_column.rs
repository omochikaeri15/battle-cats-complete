use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

const DEFAULT_GAP: f32 = 4.0;
const DEFAULT_FLOOR: f32 = 8.0;

fn shrink_by(lines: &[(String, f32)], widths: &[f32], available: f32, floor: f32) -> f32 {
    let deepest = lines.iter().map(|(_, size)| size - floor).fold(0.0, f32::max);

    if available <= 0.0 || deepest <= 0.0 {
        return 0.0;
    }

    let mut shrink: f32 = 0.0;

    for ((_, size), width) in lines.iter().zip(widths) {
        if *width <= available {
            continue;
        }

        shrink = shrink.max(size - size * available / width);
    }

    shrink.min(deepest)
}

#[component]
pub(crate) fn FitColumn(lines: Vec<(String, f32)>, #[props(default = DEFAULT_GAP)] gap: f32, #[props(default = DEFAULT_FLOOR)] floor: f32, #[props(default)] bold_first: bool) -> Element {
    let mut available = use_signal(|| 0.0f32);
    let mut widths: Signal<Vec<f32>> = use_signal(Vec::new);
    let mut seen = use_signal(String::new);
    let probes: Rc<RefCell<Vec<(usize, MountedEvent)>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let mut probed = use_signal(|| 0usize);
    let stamp = lines.iter().map(|(text, size)| format!("{text}\u{1f}{size}")).collect::<Vec<_>>().join("\u{1e}");

    if *seen.peek() != stamp {
        seen.set(stamp);
        widths.set(Vec::new());
        probes.borrow_mut().clear();
        probed.set(0);
    }

    let total = lines.len();
    let gathered = Rc::clone(&probes);

    use_effect(move || {
        if probed() < total || !widths.peek().is_empty() || total == 0 {
            return;
        }

        let held: Vec<(usize, MountedEvent)> = gathered.borrow().clone();

        spawn(async move {
            let mut measured = vec![0.0f32; total];

            for (index, probe) in held {
                if let Ok(rect) = probe.get_client_rect().await
                    && let Some(slot) = measured.get_mut(index)
                {
                    *slot = rect.size.width as f32;
                }
            }

            widths.set(measured);
        });
    });

    let known = widths();
    let shrink = if known.len() == total { shrink_by(&lines, &known, available(), floor) } else { 0.0 };
    let ready = known.len() == total;
    let visibility = if ready { "visible" } else { "hidden" };

    rsx! {
        div {
            class: "fit-column",
            style: "gap: {gap}px; visibility: {visibility};",
            onresize: move |event| {
                if let Ok(size) = event.data().get_content_box_size() {
                    available.set(size.width as f32);
                }
            },
            for (index, (text, size)) in lines.iter().enumerate() {
                div { key: "{index}", class: if bold_first && index == 0 { "fit-line bold" } else { "fit-line" }, style: "font-size: {(size - shrink).max(floor)}px;", "{text}" }
            }
            if !ready {
                for (index, (text, size)) in lines.iter().enumerate() {
                    span {
                        key: "probe-{index}",
                        class: if bold_first && index == 0 { "fit-probe bold" } else { "fit-probe" },
                        style: "font-size: {size}px;",
                        onmounted: {
                            let probes = Rc::clone(&probes);

                            move |event: MountedEvent| {
                                probes.borrow_mut().push((index, event));
                                probed.set(probes.borrow().len());
                            }
                        },
                        "{text}"
                    }
                }
            }
        }
    }
}
