use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use futures_channel::mpsc::unbounded;
use futures_util::StreamExt;
use tracing::debug;

use kokoro::domains::sandbox::keybind::Bind;

use crate::host::{Action, Bounds, Canvas, Diagnostics, Phase, Touch};
use crate::platform::Platform;
use crate::platform::battle::{BattleCommand, BattleEvent, Standing};
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
    round: Signal<u32>,
}

impl Battle {
    pub(crate) fn notify(self, platform: &Platform, text: String) {
        let mut battle = self;
        let round = *battle.round.peek() + 1;

        battle.round.set(round);
        battle.notice.set(Some(text));

        let sleeper = platform.clone();

        spawn(async move {
            sleeper.host().sleep(NOTICE_MS).await;

            if *battle.round.peek() == round {
                battle.notice.set(None);
            }
        });
    }

    fn report(self, platform: &Platform, text: String) {
        let mut battle = self;
        let round = *battle.round.peek() + 1;

        battle.round.set(round);
        battle.status.set(Some(text));

        let sleeper = platform.clone();

        spawn(async move {
            sleeper.host().sleep(STATUS_MS).await;

            if *battle.round.peek() == round {
                battle.status.set(None);
            }
        });
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
        round: Signal::new(0),
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
                    BattleEvent::Warming { done, total } => battle.warming.set((done < total).then_some((done, total))),
                    BattleEvent::Frame(_) | BattleEvent::Sheets(_) | BattleEvent::Audio(_) => {}
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
    let ratio = platform.host().pixel_ratio();
    let (pixel_width, pixel_height) = ((width * ratio).round().max(1.0) as u32, (height * ratio).round().max(1.0) as u32);

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
                width: "{pixel_width}",
                height: "{pixel_height}",
                onmounted: move |_| attacher.host().battle_canvas(CANVAS_ID),
                oncontextmenu: move |event| event.prevent_default(),
                onpointerdown: move |event| {
                    if covered || frozen {
                        return;
                    }

                    let point = event.data().element_coordinates();

                    if let Some((x, y)) = locate(point.x, point.y) {
                        pressed.set(true);
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
