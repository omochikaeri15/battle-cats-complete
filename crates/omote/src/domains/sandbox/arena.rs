use std::rc::Rc;
use std::sync::Arc;

use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use futures_channel::mpsc::unbounded;
use futures_util::StreamExt;
use tracing::{debug, warn};

use kokoro::domains::sandbox::keybind::Bind;

use crate::host::{Action, Bounds, Canvas, Diagnostics, Phase, Touch};
use crate::platform::Platform;
use crate::platform::battle::{BattleCommand, BattleEvent, Filmed, Filming, Recorded, Standing};
use crate::systems::context;

use super::state::{self, Device, Volume};

const CANVAS_ID: &str = "battle-canvas";
const SPEED_UP_ITEM: i32 = 0;
const CAT_CPU_ITEM: i32 = 3;
const SNIPER_ITEM: i32 = 5;
const WHEEL_NOTCH_PIXELS: f32 = 100.0;
const WHEEL_NOTCH_LINES: f32 = 3.0;
const PAGE_LINES: f32 = 10.0;
const SPREAD_PER_LINE: f32 = 28.0;
const RESTART_WINDOW_MS: f64 = 400.0;
const NOTICE_MS: u32 = 2000;
const STATUS_MS: u32 = 2000;
const CUT_MS: u32 = 3000;
const ENDED_MS: u32 = 2000;
const SETTLED_MS: u32 = 3000;
const BUNDLE_NAME: &str = "Replay.bcv";
const BUNDLE_MIME: &str = "application/octet-stream";
const DONE_LABEL: &str = "Done";
const EXPORTING_STAGE: &str = "Exporting";
const UPLOADING_STAGE: &str = "Uploading";
const RENDERING_STAGE: &str = "Rendering";
const SOUNDING_STAGE: &str = "Adding Sound";
const ABORTING_LABEL: &str = "Aborting...";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Video {
    Idle,
    Running,
    Aborting,
    Done,
    Failed,
    Terminated,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Work {
    Idle,
    Running,
    Done,
    Failed,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Status {
    pub(crate) label: String,
    pub(crate) ratio: f32,
}

impl Status {
    fn at(ratio: f32, label: String) -> Self {
        Self { ratio, label }
    }

    fn progress(stage: &str, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);

        Self::at(ratio, format!("{stage} | {}%", (ratio * 100.0) as i32))
    }
}
const SPACE: &str = " ";
const SPACE_NAME: &str = "Space";

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Battle {
    pub(crate) standing: Signal<Standing>,
    pub(crate) failure: Signal<Option<String>>,
    pub(crate) status: Signal<Option<String>>,
    pub(crate) warming: Signal<Option<(usize, usize)>>,
    pub(crate) notice: Signal<Option<String>>,
    pub(crate) diagnostics: Signal<Option<Diagnostics>>,
    pub(crate) diagnosing: Signal<bool>,
    pub(crate) latest: Signal<Option<Rc<Recorded>>>,
    pub(crate) exporting: Signal<Work>,
    pub(crate) uploading: Signal<Work>,
    pub(crate) video: Signal<Video>,
    pub(crate) job: Signal<Option<Status>>,
    pub(crate) foreign: Signal<Option<String>>,
    pub(crate) cut: Signal<bool>,
    notice_round: Signal<u32>,
    status_round: Signal<u32>,
    cut_round: Signal<u32>,
    export_round: Signal<u32>,
    upload_round: Signal<u32>,
    video_round: Signal<u32>,
    job_round: Signal<u32>,
}

fn hold<T: Clone + 'static>(platform: &Platform, mut slot: Signal<T>, mut round: Signal<u32>, shown: T, resting: T, millis: u32) {
    let next = *round.peek() + 1;

    round.set(next);
    slot.set(shown);

    let sleeper = platform.clone();

    spawn(async move {
        sleeper.host().sleep(millis).await;

        if *round.peek() == next {
            slot.set(resting);
        }
    });
}

impl Battle {
    pub(crate) fn notify(self, platform: &Platform, text: String) {
        hold(platform, self.notice, self.notice_round, Some(text), None, NOTICE_MS);
    }

    pub(crate) fn report(self, platform: &Platform, text: String) {
        hold(platform, self.status, self.status_round, Some(text), None, STATUS_MS);
    }

    fn progress(self, stage: &str, ratio: f32) {
        let (mut job, mut round) = (self.job, self.job_round);

        let next = *round.peek() + 1;

        round.set(next);
        job.set(Some(Status::progress(stage, ratio)));
    }

    fn settle(self, platform: &Platform, slot: Signal<Work>, round: Signal<u32>, outcome: Result<(), String>) {
        let (ended, status) = match outcome {
            Ok(()) => (Work::Done, Status::at(1.0, DONE_LABEL.to_owned())),
            Err(reason) => (Work::Failed, Status::at(1.0, format!("Failed | {reason}"))),
        };

        hold(platform, slot, round, ended, Work::Idle, ENDED_MS);
        hold(platform, self.job, self.job_round, Some(status), None, SETTLED_MS);
    }

    pub(crate) fn export(self, platform: &Platform) {
        let (mut exporting, mut round) = (self.exporting, self.export_round);
        let next = *round.peek() + 1;

        round.set(next);
        exporting.set(Work::Running);
        self.progress(EXPORTING_STAGE, 0.0);
        platform.host().battle(BattleCommand::ExportLatest);
    }

    pub(crate) fn upload(self, platform: &Platform, name: String, bytes: Vec<u8>) {
        let (mut uploading, mut round) = (self.uploading, self.upload_round);
        let next = *round.peek() + 1;

        round.set(next);
        uploading.set(Work::Running);
        self.progress(UPLOADING_STAGE, 0.0);
        platform.host().battle(BattleCommand::Upload { name, bytes });
    }

    pub(crate) fn unreadable(self, platform: &Platform, reason: String) {
        warn!("A replay could not be read for uploading: {reason}");
        self.settle(platform, self.uploading, self.upload_round, Err(reason));
    }

    pub(crate) fn delete(self, platform: &Platform) {
        platform.host().battle(BattleCommand::DeleteLatest);
    }

    pub(crate) fn film(self, platform: &Platform, filming: Filming) {
        let (mut video, mut round) = (self.video, self.video_round);
        let next = *round.peek() + 1;

        round.set(next);
        video.set(Video::Running);
        self.progress(RENDERING_STAGE, 0.0);
        platform.host().battle(BattleCommand::ExportVideo(Box::new(filming)));
    }

    pub(crate) fn abort_film(self, platform: &Platform) {
        let (mut video, mut job) = (self.video, self.job);
        let ratio = job.peek().as_ref().map_or(0.0, |status| status.ratio);

        video.set(Video::Aborting);
        job.set(Some(Status::at(ratio, ABORTING_LABEL.to_owned())));
        platform.host().battle(BattleCommand::AbortVideo);
    }

    fn filming(self, stage: &str, ratio: f32) {
        if *self.video.peek() == Video::Running {
            self.progress(stage, ratio);
        }
    }

    fn filmed(self, platform: &Platform, filmed: Filmed) {
        let ended = match filmed {
            Filmed::Done => {
                hold(platform, self.job, self.job_round, Some(Status::at(1.0, DONE_LABEL.to_owned())), None, SETTLED_MS);
                Video::Done
            }
            Filmed::Failed(reason) => {
                hold(platform, self.job, self.job_round, Some(Status::at(1.0, format!("Failed | {reason}"))), None, SETTLED_MS);
                Video::Failed
            }
            Filmed::Aborted => {
                let (mut job, mut round) = (self.job, self.job_round);
                let next = *round.peek() + 1;

                round.set(next);
                job.set(None);
                Video::Terminated
            }
        };

        hold(platform, self.video, self.video_round, ended, Video::Idle, ENDED_MS);
    }
}

pub(crate) fn provide(platform: &Platform) -> Battle {
    let battle = use_context_provider(|| Battle {
        standing: Signal::new(Standing { phase: Phase::Idle, covered: false, surfaced: false }),
        failure: Signal::new(None),
        status: Signal::new(None),
        warming: Signal::new(None),
        notice: Signal::new(None),
        diagnostics: Signal::new(None),
        diagnosing: Signal::new(false),
        latest: Signal::new(None),
        exporting: Signal::new(Work::Idle),
        uploading: Signal::new(Work::Idle),
        video: Signal::new(Video::Idle),
        job: Signal::new(None),
        foreign: Signal::new(None),
        cut: Signal::new(false),
        notice_round: Signal::new(0),
        status_round: Signal::new(0),
        cut_round: Signal::new(0),
        export_round: Signal::new(0),
        upload_round: Signal::new(0),
        video_round: Signal::new(0),
        job_round: Signal::new(0),
    });
    let sandbox = state::state();
    let platform = platform.clone();

    use_hook(move || {
        let (sender, mut events) = unbounded::<BattleEvent>();

        platform.host().battle_watch(Box::new(move |event| {
            if sender.unbounded_send(event).is_err() {
                debug!("A battle event arrived after the sandbox page was gone");
            }
        }));

        spawn(async move {
            let mut battle = battle;

            while let Some(event) = events.next().await {
                match event {
                    BattleEvent::Standing(standing) => {
                        let failed = battle.failure.peek().clone();
                        let was_faulted = battle.standing.peek().phase == Phase::Faulted;

                        if standing.phase != Phase::Faulted
                            && let Some(text) = failed
                        {
                            battle.failure.set(None);

                            if !was_faulted {
                                battle.report(&platform, text);
                            }
                        }

                        if standing.phase == Phase::Idle {
                            battle.warming.set(None);
                            battle.diagnostics.set(None);
                            battle.diagnosing.set(false);
                        }

                        battle.standing.set(standing);
                    }
                    BattleEvent::Failure(text) => battle.failure.set(Some(text)),
                    BattleEvent::Options(options) => state::apply(&platform, sandbox, |next| {
                        next.music_volume = options.music;
                        next.effects_volume = options.effects;
                        next.two_rows = options.two_rows;
                        next.vibrate = options.vibrate;
                    }),
                    BattleEvent::Diagnostics(probe) => battle.diagnostics.set(Some(probe)),
                    BattleEvent::Latest(found) => battle.latest.set(found.map(|recorded| Rc::new(*recorded))),
                    BattleEvent::Exporting(share) => battle.progress(EXPORTING_STAGE, share),
                    BattleEvent::Exported(exported) => {
                        let outcome = exported.map(|bytes| platform.host().download(BUNDLE_NAME, BUNDLE_MIME, Arc::from(bytes)));

                        if let Err(reason) = &outcome {
                            warn!("The latest battle could not be exported: {reason}");
                        }

                        battle.settle(&platform, battle.exporting, battle.export_round, outcome);
                    }
                    BattleEvent::Uploading(share) => battle.progress(UPLOADING_STAGE, share),
                    BattleEvent::Uploaded(uploaded) => battle.settle(&platform, battle.uploading, battle.upload_round, uploaded),
                    BattleEvent::Foreign(version) => battle.foreign.set(Some(version)),
                    BattleEvent::Rejected(reason) => battle.report(&platform, reason),
                    BattleEvent::Cut => hold(&platform, battle.cut, battle.cut_round, true, false, CUT_MS),
                    BattleEvent::Warming { done, total } => battle.warming.set((done < total).then_some((done, total))),
                    BattleEvent::Rendering(share) => battle.filming(RENDERING_STAGE, share),
                    BattleEvent::Sounding(share) => battle.filming(SOUNDING_STAGE, share),
                    BattleEvent::Filmed(filmed) => battle.filmed(&platform, filmed),
                    BattleEvent::Frame(_) | BattleEvent::Sheets(_) | BattleEvent::Audio(_) | BattleEvent::Rolled(_) => {}
                }
            }
        });
    });

    battle
}

pub(crate) fn battle() -> Battle {
    use_context::<Battle>()
}

fn key_name(key: &Key) -> Option<String> {
    match key {
        Key::Character(typed) if typed == SPACE => Some(SPACE_NAME.to_owned()),
        Key::Character(typed) => Some(typed.to_lowercase()),
        Key::Unidentified => None,
        named => Some(format!("{named:?}")),
    }
}

#[component]
pub(crate) fn Arena() -> Element {
    let platform = use_context::<Platform>();
    let settings = context::settings();
    let sandbox = state::state();
    let battle = battle();
    let mut size = use_signal(|| (0f32, 0f32));
    let mut pressed = use_signal(|| false);
    let mut tapped: Signal<Option<f64>> = use_signal(|| None);
    let detacher = platform.clone();

    use_drop(move || detacher.host().battle_canvas(""));

    let display = platform.clone();

    use_effect(move || {
        let held = sandbox.read();

        display.host().battle_display(held.render_scale(), held.padded());
    });

    let devicer = platform.clone();

    use_effect(move || {
        let held = sandbox.read();

        devicer.host().battle(BattleCommand::Phone(held.device == Device::Phone));
        devicer.host().battle(BattleCommand::Volumes(Volume::nearest(held.music_volume).percent(), Volume::nearest(held.effects_volume).percent()));
    });

    let standing = (battle.standing)();
    let idle = standing.phase == Phase::Idle;
    let mut layer: Signal<Option<MountedEvent>> = use_signal(|| None);

    use_effect(move || {
        let phase = (battle.standing)().phase;

        if phase == Phase::Covering
            && let Some(mounted) = layer.peek().clone()
        {
            spawn(async move {
                let _ = mounted.set_focus(true).await;
            });
        }
    });

    let frozen = standing.phase == Phase::Faulted;
    let covered = standing.covered;
    let held = sandbox.read();
    let canvas = Canvas { surfaced: standing.surfaced, scale: held.render_scale(), padded: held.padded() };

    drop(held);

    let (width, height) = size();
    let bounds = Bounds { width, height };
    let locate = move |x: f64, y: f64| -> Option<(i32, i32)> {
        let (x, y) = canvas.unpad(x as f32, y as f32, bounds);

        ((0.0..=bounds.width).contains(&x) && (0.0..=bounds.height).contains(&y)).then(|| (x.round() as i32, y.round() as i32))
    };
    let clamp = move |x: f64, y: f64| -> (i32, i32) {
        let (x, y) = canvas.unpad(x as f32, y as f32, bounds);

        (x.clamp(0.0, bounds.width).round() as i32, y.clamp(0.0, bounds.height).round() as i32)
    };
    let toucher = platform.clone();
    let touch = move |touch: Touch| toucher.host().battle(BattleCommand::Touches(vec![touch]));
    let touch_down = touch.clone();
    let touch_move = touch.clone();
    let touch_up = touch.clone();
    let touch_cancel = touch.clone();
    let touch_lost = touch.clone();
    let capturer = platform.clone();
    let keyer = platform.clone();
    let mut key = move |key: &Key, pressed_now: bool, repeat: bool| {
        let Some(name) = key_name(key) else {
            return;
        };
        let Some(bind) = settings.read().sandbox.keys.bound(&name) else {
            return;
        };
        let action = match bind {
            Bind::Slot(slot) => Action::Slot(i32::from(slot)),
            Bind::SpeedUp => Action::Item(SPEED_UP_ITEM),
            Bind::CatCpu => Action::Item(CAT_CPU_ITEM),
            Bind::Sniper => Action::Item(SNIPER_ITEM),
            Bind::Worker => Action::Worker,
            Bind::Cannon => Action::Cannon,
            Bind::ZoomOut => Action::ZoomOut,
            Bind::ZoomIn => Action::ZoomIn,
            Bind::Left => Action::PanLeft,
            Bind::Right => Action::PanRight,
            Bind::Pause => Action::Pause,
            Bind::Diagnostics => {
                if pressed_now && !repeat {
                    let on = !*battle.diagnosing.peek();
                    let mut battle = battle;

                    battle.diagnosing.set(on);
                    keyer.host().battle(BattleCommand::Diagnose(on));
                }

                return;
            }
            Bind::Quality => {
                if pressed_now && !repeat {
                    state::apply(&keyer, sandbox, |next| next.cycle_quality());
                    battle.notify(&keyer, format!("Image Quality {}%", sandbox.peek().quality()));
                }

                return;
            }
            Bind::Restart => {
                if pressed_now && !repeat {
                    let now = keyer.host().now();
                    let doubled = tapped.take().is_some_and(|at| now - at <= RESTART_WINDOW_MS);

                    if doubled {
                        keyer.host().battle(BattleCommand::Restart);
                    } else {
                        tapped.set(Some(now));
                    }
                }

                return;
            }
        };

        if pressed_now && repeat {
            return;
        }

        keyer.host().battle(BattleCommand::Key(action, pressed_now));
    };
    let mut key_down = key.clone();
    let attacher = platform.clone();
    let sizer = platform.clone();

    rsx! {
        div {
            class: match (idle, standing.surfaced) {
                (true, _) => "battle-layer idle",
                (false, true) => "battle-layer surfaced",
                (false, false) => "battle-layer",
            },
            tabindex: "0",
            onmounted: move |event| {
                let platform = sizer.clone();

                layer.set(Some(event.clone()));

                spawn(async move {
                    if let Ok(rect) = event.data().get_client_rect().await {
                        size.set((rect.size.width as f32, rect.size.height as f32));
                        platform.host().battle(BattleCommand::Resize(rect.size.width as f32, rect.size.height as f32));
                    }
                });
            },
            onresize: move |event| {
                if let Ok(box_size) = event.data().get_content_box_size() {
                    size.set((box_size.width as f32, box_size.height as f32));
                    platform.host().battle(BattleCommand::Resize(box_size.width as f32, box_size.height as f32));
                }
            },
            onkeydown: move |event| {
                event.prevent_default();
                key_down(&event.data().key(), true, event.data().is_auto_repeating());
            },
            onkeyup: move |event| {
                event.prevent_default();
                key(&event.data().key(), false, false);
            },
            canvas {
                id: CANVAS_ID,
                class: "battle-canvas",
                onmounted: move |_| attacher.host().battle_canvas(CANVAS_ID),
                oncontextmenu: move |event| event.prevent_default(),
                onpointerdown: move |event| {
                    if covered || frozen || event.data().trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }

                    let point = event.data().element_coordinates();

                    if let Some((x, y)) = locate(point.x, point.y) {
                        pressed.set(true);
                        capturer.host().capture_pointer(CANVAS_ID, event.data().pointer_id());
                        touch_down(Touch::Pressed { x, y });
                    }
                },
                onpointermove: move |event| {
                    let point = event.data().element_coordinates();

                    if pressed() {
                        let (x, y) = clamp(point.x, point.y);

                        touch_move(Touch::Moved { x, y });
                    } else if !covered
                        && !frozen
                        && let Some((x, y)) = locate(point.x, point.y)
                    {
                        touch_move(Touch::Moved { x, y });
                    }
                },
                onpointerup: move |_| {
                    if pressed() {
                        pressed.set(false);
                        touch_up(Touch::Released);
                    }
                },
                onpointercancel: move |_| {
                    if pressed() {
                        pressed.set(false);
                        touch_cancel(Touch::Released);
                    }
                },
                onlostpointercapture: move |_| {
                    if pressed() {
                        pressed.set(false);
                        touch_lost(Touch::Released);
                    }
                },
                onwheel: move |event| {
                    event.prevent_default();

                    if covered || frozen {
                        return;
                    }

                    let lines = match event.data().delta() {
                        WheelDelta::Pixels(vector) => -(vector.y as f32) / WHEEL_NOTCH_PIXELS,
                        WheelDelta::Lines(vector) => -(vector.y as f32) / WHEEL_NOTCH_LINES,
                        WheelDelta::Pages(vector) => -(vector.y as f32) * PAGE_LINES,
                    };

                    touch(Touch::Pinched { spread: (lines * SPREAD_PER_LINE) as i32 });
                },
            }
        }
    }
}
