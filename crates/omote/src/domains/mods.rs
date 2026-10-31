use std::rc::Rc;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use tracing::warn;

use kokoro::domains::sandbox::replay::VANILLA_APP;

use crate::platform::Platform;
use crate::platform::mods::{ModCommand, ModInfo, ModState, Mounted};
use crate::systems::animation::{self, Rigs};
use crate::systems::catalog::{self, Catalogs};
use crate::systems::context;
use crate::systems::images::{self, Pictures};
use crate::systems::library::Library;
use crate::widget::Status;

const MOD_LIMIT: u64 = 256 * 1024 * 1024;
const BCM_ACCEPT: &str = ".bcm";
const BCM_SUFFIX: &str = ".bcm";
const ENDED_MS: u32 = 2000;
const CONFIRM_MS: u32 = 2000;
const NOTICE_MS: u32 = 3000;
const TITLE_HINT: &str = "Mod Title";
const ENABLE_LABEL: &str = "Enable Mod";
const DISABLE_LABEL: &str = "Disable Mod";
const UPLOAD_LABEL: &str = "Upload Mod";
const UPLOADING_LABEL: &str = "Uploading...";
const UPLOADED_LABEL: &str = "Uploaded!";
const FAILED_LABEL: &str = "Failed!";
const LARGE_LABEL: &str = "Too Large!";
const DELETE_LABEL: &str = "Delete Mod";
const CONFIRM_LABEL: &str = "Are You Sure?";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Upload {
    Idle,
    Running,
    Done,
    Failed,
    Large,
}

#[derive(Clone, Copy)]
pub(crate) struct ModsState {
    held: Signal<Option<Rc<ModInfo>>>,
    mounted: Signal<Option<Mounted>>,
    asked: Signal<bool>,
    busy: Signal<bool>,
    uploading: Signal<Upload>,
    upload_round: Signal<u32>,
    notice: Signal<Option<String>>,
    notice_round: Signal<u32>,
}

pub(crate) fn provide() -> ModsState {
    use_context_provider(|| ModsState {
        held: Signal::new(None),
        mounted: Signal::new(None),
        asked: Signal::new(false),
        busy: Signal::new(false),
        uploading: Signal::new(Upload::Idle),
        upload_round: Signal::new(0),
        notice: Signal::new(None),
        notice_round: Signal::new(0),
    })
}

pub(crate) fn state() -> ModsState {
    use_context::<ModsState>()
}

#[derive(Clone)]
struct Caches {
    library: Library,
    catalogs: Catalogs,
    pictures: Pictures,
    rigs: Rigs,
}

fn hold<T: Clone + 'static>(platform: &Platform, mut slot: Signal<T>, mut round: Signal<u32>, shown: T, resting: T, millis: u32) {
    let next = *round.peek() + 1;

    round.set(next);
    slot.set(shown);

    let sleeper = platform.clone();

    spawn_forever(async move {
        sleeper.host().sleep(millis).await;

        if *round.peek() == next {
            slot.set(resting);
        }
    });
}

fn bundle_name(file: &str) -> String {
    let stem = file.len().checked_sub(BCM_SUFFIX.len()).filter(|&cut| file.get(cut..).is_some_and(|tail| tail.eq_ignore_ascii_case(BCM_SUFFIX))).and_then(|cut| file.get(..cut));

    stem.unwrap_or(file).to_owned()
}

impl ModsState {
    pub(crate) fn app(&self) -> String {
        if self.mounted.peek().is_none() {
            return VANILLA_APP.to_owned();
        }

        self.held.peek().as_ref().map_or_else(|| VANILLA_APP.to_owned(), |info| if info.title.is_empty() { info.name.clone() } else { info.title.clone() })
    }

    fn notify(self, platform: &Platform, text: String) {
        hold(platform, self.notice, self.notice_round, Some(text), None, NOTICE_MS);
    }

    fn adopt(self, caches: &Caches, received: ModState) {
        let (mut held, mut mounted) = (self.held, self.mounted);
        let mut library = caches.library;
        let mut catalogs = caches.catalogs;

        held.set(received.held.map(Rc::new));

        if *mounted.peek() == received.mounted {
            return;
        }

        library.remount(received.mounted.as_ref());
        mounted.set(received.mounted);
        catalogs.reset();
        caches.pictures.forget_all();
        caches.rigs.forget_all();
    }

    fn run(self, platform: &Platform, caches: &Caches, command: ModCommand, ended: impl FnOnce(&Platform, Result<(), String>) + 'static) {
        let mut busy = self.busy;
        let pending = platform.host().mods(command, Box::new(|_| {}));
        let (platform, caches) = (platform.clone(), caches.clone());

        busy.set(true);

        spawn_forever(async move {
            let outcome = pending.await.map(|received| self.adopt(&caches, received)).map_err(|fault| fault.to_string());

            busy.set(false);

            if let Err(reason) = &outcome {
                warn!("A mod command failed: {reason}");
                self.notify(&platform, format!("Failed | {reason}"));
            }

            ended(&platform, outcome);
        });
    }
}

#[component]
pub(crate) fn Mods() -> Element {
    let platform = use_context::<Platform>();
    let library = use_context::<Library>();
    let globals = context::globals();
    let state = state();
    let caches = Caches { library, catalogs: catalog::catalogs(), pictures: images::pictures(), rigs: animation::rigs() };
    let mut deleting = use_signal(|| false);
    let mut delete_round = use_signal(|| 0u32);
    let ready = globals.is_some();
    let asker = platform.clone();
    let asking = caches.clone();

    use_effect(use_reactive!(|ready| {
        let mut asked = state.asked;

        if ready && !*asked.peek() {
            asked.set(true);
            state.run(&asker, &asking, ModCommand::Held, |_, _| {});
        }
    }));

    if let Some(fault) = library.fault() {
        return rsx! { Status { title: "The file host could not be reached", detail: fault.to_string() } };
    }

    if !ready {
        let phase = library.phase();

        return rsx! { Status { title: phase.label.to_string(), detail: Library::detail(phase.done, phase.total) } };
    }

    let held = (state.held)();
    let enabled = (state.mounted)().is_some();
    let busy = (state.busy)();
    let uploading = (state.uploading)();
    let present = held.is_some();
    let info = held.as_deref().cloned().unwrap_or_default();
    let (toggle_label, toggle_class) = if enabled { (DISABLE_LABEL, "btn btn-danger mods-action") } else { (ENABLE_LABEL, "btn btn-success mods-action") };
    let (upload_label, upload_class) = match uploading {
        Upload::Idle => (UPLOAD_LABEL, "btn btn-primary mods-action"),
        Upload::Running => (UPLOADING_LABEL, "btn btn-status-warning mods-action"),
        Upload::Done => (UPLOADED_LABEL, "btn btn-status-success mods-action"),
        Upload::Failed => (FAILED_LABEL, "btn btn-status-danger mods-action"),
        Upload::Large => (LARGE_LABEL, "btn btn-status-danger mods-action"),
    };
    let delete_label = if deleting() { CONFIRM_LABEL } else { DELETE_LABEL };
    let notice = (state.notice)();
    let notice_class = if notice.is_some() { "toast notice-banner" } else { "toast notice-banner hidden" };
    let notice_text = notice.unwrap_or_default();
    let toggler = platform.clone();
    let toggling = caches.clone();
    let uploader = platform.clone();
    let uploads = caches.clone();
    let deleter = platform.clone();
    let deletes = caches;

    rsx! {
        div { class: "mods-page scroll",
            input { class: "input mods-title", r#type: "text", readonly: true, placeholder: TITLE_HINT, value: "{info.title}" }
            div { class: "rule mods-rule" }
            div { class: "mods-actions",
                button {
                    class: "{toggle_class}",
                    disabled: !present || busy || uploading == Upload::Running,
                    onclick: move |_| state.run(&toggler, &toggling, if enabled { ModCommand::Disable } else { ModCommand::Enable }, |_, _| {}),
                    "{toggle_label}"
                }
                button {
                    class: "{upload_class}",
                    disabled: busy || uploading == Upload::Running,
                    onclick: move |_| {
                        let platform = uploader.clone();
                        let caches = uploads.clone();
                        let picking = platform.host().pick_file(BCM_ACCEPT, MOD_LIMIT);

                        spawn_forever(async move {
                            let picked = match picking.await {
                                Ok(Some(picked)) => picked,
                                Ok(None) => return,
                                Err(fault) => {
                                    warn!("A mod could not be read for uploading: {fault}");
                                    state.notify(&platform, format!("Failed | the file could not be read: {fault}"));
                                    hold(&platform, state.uploading, state.upload_round, Upload::Failed, Upload::Idle, ENDED_MS);

                                    return;
                                }
                            };
                            let Some(bytes) = picked.bytes else {
                                hold(&platform, state.uploading, state.upload_round, Upload::Large, Upload::Idle, ENDED_MS);

                                return;
                            };
                            let mut uploading = state.uploading;
                            let mut round = state.upload_round;
                            let next = *round.peek() + 1;

                            round.set(next);
                            uploading.set(Upload::Running);
                            state.run(&platform, &caches, ModCommand::Upload { name: bundle_name(&picked.name), bytes: bytes.to_vec() }, move |platform, outcome| {
                                let ended = if outcome.is_ok() { Upload::Done } else { Upload::Failed };

                                hold(platform, state.uploading, state.upload_round, ended, Upload::Idle, ENDED_MS);
                            });
                        });
                    },
                    "{upload_label}"
                }
                button {
                    class: "btn btn-danger mods-action",
                    disabled: !present || enabled || busy || uploading == Upload::Running,
                    onclick: move |_| {
                        let next = *delete_round.peek() + 1;

                        delete_round.set(next);

                        if !deleting() {
                            deleting.set(true);

                            let sleeper = deleter.clone();

                            spawn(async move {
                                sleeper.host().sleep(CONFIRM_MS).await;

                                if *delete_round.peek() == next {
                                    deleting.set(false);
                                }
                            });

                            return;
                        }

                        deleting.set(false);
                        state.run(&deleter, &deletes, ModCommand::Delete, |_, _| {});
                    },
                    "{delete_label}"
                }
            }
            div { class: "rule mods-rule" }
            div { class: "card-outlined mods-card",
                div { class: "section",
                    div { class: "section-title", "Information" }
                    div { class: "rule" }
                    for (label, value) in [("Author", info.author.clone()), ("Version", info.version.clone()), ("Package", info.package.clone())] {
                        div { key: "{label}", class: "mods-field",
                            span { class: "mods-label", "{label}" }
                            input { class: "input mods-value", r#type: "text", readonly: true, value: "{value}" }
                        }
                    }
                }
            }
            div { class: "card-outlined mods-card mods-description",
                div { class: "section",
                    div { class: "section-title", "Description" }
                    div { class: "rule" }
                    textarea { class: "input mods-text", readonly: true, value: "{info.description}" }
                }
            }
            div { class: "toast-slot",
                div { class: "{notice_class}", "{notice_text}" }
            }
        }
    }
}
