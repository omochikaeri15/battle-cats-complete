use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use dioxus::prelude::*;
use image::{ImageFormat, RgbaImage};
use tracing::error;

use kokoro::common::formats::SpriteSheet;
use kokoro::systems::statblock::{self, Fonts, StatblockData};

use crate::platform::{Bytes, Fault, Platform, Script};
use crate::systems::library::Library;

const FEEDBACK_MS: u32 = 2000;
const ARRIVAL_POLL_MS: u32 = 50;
const ARRIVAL_LIMIT_MS: u32 = 10_000;

pub(crate) struct Statblock {
    pub(crate) data: StatblockData,
    pub(crate) sheets: Rc<Vec<SpriteSheet>>,
    pub(crate) priority: Vec<String>,
}

struct FontSet {
    jp: Bytes,
    kr: Bytes,
    tc: Bytes,
    th: Bytes,
}

impl FontSet {
    fn fonts(&self) -> Fonts<'_> {
        Fonts { jp: &self.jp, kr: &self.kr, tc: &self.tc, th: &self.th }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Typefaces(Rc<RefCell<Option<Rc<FontSet>>>>);

pub(crate) fn provide() -> Typefaces {
    use_context_provider(Typefaces::default)
}

async fn fonts(platform: &Platform, typefaces: &Typefaces) -> Result<Rc<FontSet>, Fault> {
    if let Some(held) = typefaces.0.borrow().as_ref() {
        return Ok(Rc::clone(held));
    }

    let host = platform.host();
    let set = Rc::new(FontSet { jp: host.font(Script::Japanese).await?, kr: host.font(Script::Korean).await?, tc: host.font(Script::TraditionalChinese).await?, th: host.font(Script::Thai).await? });

    *typefaces.0.borrow_mut() = Some(Rc::clone(&set));

    Ok(set)
}

pub(crate) fn png(image: &RgbaImage) -> Result<Bytes, Fault> {
    let mut bytes = Vec::new();

    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).map_err(|err| Fault(format!("the image could not be encoded: {err}")))?;

    Ok(Bytes::from(bytes))
}

fn file_name(data: &StatblockData) -> String {
    let safe: String = data.top_value.chars().filter(|c| c.is_alphanumeric() || *c == '+').collect();
    let prefix = if data.is_cat { "Lv" } else { "Mag" };

    format!("{}.{prefix}{safe}.statblock.png", data.id_str)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Copy,
    Save,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Busy,
    Done(bool),
}

pub(crate) fn label(phase: Phase, idle: &str, busy: &str, success: &str) -> String {
    match phase {
        Phase::Idle => idle.to_owned(),
        Phase::Busy => busy.to_owned(),
        Phase::Done(true) => success.to_owned(),
        Phase::Done(false) => "Failed!".to_owned(),
    }
}

pub(crate) fn class(phase: Phase) -> &'static str {
    match phase {
        Phase::Idle => "btn export-btn idle",
        Phase::Busy => "btn export-btn busy",
        Phase::Done(true) => "btn export-btn ok",
        Phase::Done(false) => "btn export-btn bad",
    }
}

async fn arrived(platform: &Platform, library: Library) {
    let mut waited = 0;

    while library.waiting() && waited < ARRIVAL_LIMIT_MS {
        platform.host().sleep(ARRIVAL_POLL_MS).await;
        waited += ARRIVAL_POLL_MS;
    }
}

pub(crate) async fn settle(platform: Platform, mut phase: Signal<Phase>, ok: bool) {
    phase.set(Phase::Done(ok));
    platform.host().sleep(FEEDBACK_MS).await;

    if *phase.peek() == Phase::Done(ok) {
        phase.set(Phase::Idle);
    }
}

#[component]
pub(crate) fn ExportButtons(request: Callback<(), Option<Statblock>>) -> Element {
    let platform = use_context::<Platform>();
    let typefaces = use_context::<Typefaces>();
    let library = use_context::<Library>();
    let copy_phase = use_signal(|| Phase::Idle);
    let save_phase = use_signal(|| Phase::Idle);
    let can_copy = platform.host().can_copy();
    let busy = copy_phase() == Phase::Busy || save_phase() == Phase::Busy;
    let run = use_callback(move |action: Action| {
        let platform = platform.clone();
        let typefaces = typefaces.clone();
        let mut phase = if action == Action::Copy { copy_phase } else { save_phase };

        if *copy_phase.peek() == Phase::Busy || *save_phase.peek() == Phase::Busy {
            return;
        }

        let Some(first) = request.call(()) else {
            spawn(settle(platform, phase, false));
            return;
        };

        phase.set(Phase::Busy);

        spawn(async move {
            let request = match library.waiting() {
                true => {
                    arrived(&platform, library).await;
                    request.call(()).unwrap_or(first)
                }
                false => first,
            };
            let outcome = async {
                let set = fonts(&platform, &typefaces).await?;
                let image = statblock::build_statblock_image(&request.priority, &request.sheets, request.data.clone(), &set.fonts()).map_err(Fault)?;
                let bytes = png(&image)?;

                match action {
                    Action::Copy => platform.host().copy_image(bytes).await,
                    Action::Save => {
                        platform.host().download(&file_name(&request.data), "image/png", bytes);
                        Ok(())
                    }
                }
            }
            .await;

            if let Err(fault) = &outcome {
                error!("The statblock export failed: {fault}");
            }

            settle(platform, phase, outcome.is_ok()).await;
        });
    });

    rsx! {
        div { class: "export-buttons",
            if can_copy {
                button { class: class(copy_phase()), disabled: busy, onclick: move |_| run.call(Action::Copy), {label(copy_phase(), "Copy Image", "Copying...", "Copied!")} }
            }
            button { class: class(save_phase()), disabled: busy, onclick: move |_| run.call(Action::Save), {label(save_phase(), "Export Image", "Exporting...", "Exported!")} }
        }
    }
}
