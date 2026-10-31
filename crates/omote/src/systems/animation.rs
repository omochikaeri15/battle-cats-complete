mod controls;
mod export;
mod overlay;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use dioxus::core::spawn_forever;
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use tracing::warn;

use codex::graphics::animate::{FrameData, resolve_frame};
use codex::graphics::rig::{Animation, BoundingBox, Model, Rig};
use kokoro::domains::cat::animation as cat_animation;
use kokoro::domains::enemy::animation as enemy_animation;
use kokoro::domains::settings::Settings;
use kokoro::Source;
use kokoro::systems::animation::{Loop, MotionSet, NO_OFFSET, Placement, Rigging, cycle, loop_frame, restart, restart_frame};

use crate::app::remember_settings;
use crate::domains::cat::{self, SelectedCat};
use crate::domains::enemy::SelectedEnemy;
use crate::platform::{Atlas, Painted, Platform, Scene, Surface};
use crate::systems::context::{self, Shared};
use crate::systems::library::Library;
use crate::theme::AppTheme;
use crate::widget::{Live, Toggle};

use controls::Controls;
use overlay::{Pose, View};

thread_local! {
    static VIEWERS: Cell<u32> = const { Cell::new(0) };
}

fn next_viewer() -> u32 {
    VIEWERS.with(|count| {
        let next = count.get() + 1;

        count.set(next);
        next
    })
}
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_RATE_PER_PIXEL: f32 = 0.0016;
const ZOOM_SCROLL_STRENGTH: f32 = 2.0;
const ZOOM_DECAY_RATE: f32 = 16.0;
const ZOOM_EPSILON: f32 = 0.05;
const LINE_PIXELS: f32 = 60.0;
const PAGE_LINES: f32 = 10.0;
const WHEEL_NOTCH_PIXELS: f32 = 100.0;
const WHEEL_NOTCH_LINES: f32 = 3.0;
const FRAME_ADVANCE_PER_TICK: f32 = 0.5;
const TICK_MS: f64 = 1000.0 / 60.0;
const LONGEST_TICK_MS: f64 = 100.0;
const HOLD_DELAY_MS: f64 = 200.0;
const HOLD_FRAMES_PER_SEC: f32 = 30.0;
const NO_TINT: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
const CULL_SCALE: f32 = 100.0;
const MIN_SELECTION_AREA: f32 = 25.0;
const SELECT_HINT: &str = "Drag to set the camera";
const COLUMNS: usize = 4;
const DEFAULT_SLOTS: usize = 8;
const MISSING_FILES_NOTICE: &str = "Missing essential files needed to load Entity";

#[derive(Clone, Copy)]
pub(crate) struct ViewerState {
    selected_label: Signal<Option<String>>,
    playing: Signal<bool>,
    frame: Signal<f32>,
    pan: Signal<(f32, f32)>,
    zoom: Signal<f32>,
    loop_start: Signal<Option<f32>>,
    loop_end: Signal<Option<f32>>,
    range_inputs: Signal<(String, String)>,
    placement: Signal<Placement>,
    controls_expanded: Signal<bool>,
    hold: Signal<Option<(i8, f64)>>,
    size: Signal<(f32, f32)>,
    drag: Signal<Option<(f32, f32)>>,
    active: Signal<Option<Active>>,
    zoom_pending: Signal<f32>,
    selecting: Signal<bool>,
    selection: Signal<Option<Drag>>,
    region: Signal<Option<Region>>,
    last_set: Signal<Option<String>>,
}

fn unit_of(set: &str) -> &str {
    set.split('_').next().unwrap_or(set)
}

type Drag = ((f32, f32), (f32, f32));

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Region {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
}

pub(crate) fn provide() -> ViewerState {
    use_context_provider(|| ViewerState {
        selected_label: Signal::new(None),
        playing: Signal::new(true),
        frame: Signal::new(0.0),
        pan: Signal::new((0.0, 0.0)),
        zoom: Signal::new(1.0),
        loop_start: Signal::new(None),
        loop_end: Signal::new(None),
        range_inputs: Signal::new((String::new(), String::new())),
        placement: Signal::new(Placement::default()),
        controls_expanded: Signal::new(true),
        hold: Signal::new(None),
        size: Signal::new((0.0, 0.0)),
        drag: Signal::new(None),
        active: Signal::new(None),
        zoom_pending: Signal::new(0.0),
        selecting: Signal::new(false),
        selection: Signal::new(None),
        region: Signal::new(None),
        last_set: Signal::new(None),
    })
}

fn state() -> ViewerState {
    use_context::<ViewerState>()
}

#[derive(Clone)]
pub(crate) struct Motions {
    key: String,
    set: Rc<MotionSet>,
}

impl PartialEq for Motions {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

struct Sheet {
    rig: Option<Arc<Rig>>,
    deferred: bool,
}

#[derive(Clone)]
struct Parsed(Rc<Sheet>);

impl PartialEq for Parsed {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

struct Loaded {
    rig: Option<Arc<Rig>>,
    anim: Option<Arc<Animation>>,
    failed: bool,
}

#[derive(Clone)]
struct Held(Rc<Loaded>);

impl PartialEq for Held {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
struct Active {
    model: bool,
    looping: Loop,
    anim: Option<Arc<Animation>>,
}

impl Active {
    fn loop_bound(&self) -> Option<i32> {
        self.anim.as_deref().map_or(Some(0), |anim| match self.looping {
            Loop::Exact => cycle(anim).map(|frames| frames - 1),
            Loop::Frames => Some(anim.declared_frames() - 1),
            Loop::Auto => restart(anim).map(|frames| frames - 1),
            Loop::Continuous => None,
        })
    }

    fn playback_frame(&self, frame: f32) -> f32 {
        let Some(anim) = self.anim.as_deref() else {
            return frame;
        };

        match self.looping {
            Loop::Exact | Loop::Continuous => frame,
            Loop::Frames => loop_frame(anim, frame),
            Loop::Auto => restart_frame(anim, frame),
        }
    }
}

#[derive(Clone, Copy)]
struct Request {
    slot: Option<usize>,
    trailing: bool,
}

fn place(set: &MotionSet) -> Vec<Option<usize>> {
    let requests: Vec<Request> = set.motions.iter().map(|motion| Request { slot: motion.slot, trailing: motion.file.is_none() }).collect();
    let mut slots: Vec<Option<usize>> = vec![None; DEFAULT_SLOTS];
    let settled = |request: &Request| !request.trailing;
    let mut anchored: Vec<(usize, usize)> =
        requests.iter().enumerate().filter(|(_, request)| settled(request)).filter_map(|(index, request)| Some((request.slot?, index))).collect();

    anchored.sort_by_key(|(slot, _)| *slot);

    for (slot, index) in anchored {
        settle(&mut slots, slot, 1, index);
    }

    for (index, _) in requests.iter().enumerate().filter(|(_, request)| settled(request) && request.slot.is_none()) {
        settle(&mut slots, 0, 1, index);
    }

    for (index, request) in requests.iter().enumerate().filter(|(_, request)| request.trailing) {
        settle(&mut slots, request.slot.unwrap_or(0), COLUMNS, index);
    }

    while !slots.len().is_multiple_of(COLUMNS) {
        slots.push(None);
    }

    slots
}

fn settle(slots: &mut Vec<Option<usize>>, from: usize, stride: usize, index: usize) {
    let mut at = from;

    loop {
        while at >= slots.len() {
            slots.push(None);
        }

        if slots[at].is_none() {
            slots[at] = Some(index);
            return;
        }

        at += stride;
    }
}

fn selected_index(set: &MotionSet, slots: &[Option<usize>], label: Option<&str>) -> Option<usize> {
    label.and_then(|label| set.motions.iter().position(|motion| motion.label() == label)).or_else(|| slots.iter().find_map(|slot| *slot))
}

fn offset_of(rig: Option<&Rig>, placement: Placement) -> Option<usize> {
    let rows = rig.map_or(0, |rig| rig.model.alignment.len());

    match placement {
        Placement::Bare => None,
        Placement::Row(row) if row < rows => Some(row),
        Placement::Row(_) => (rows > 0).then_some(0),
    }
}

fn offset_choices(set: &MotionSet, rig: Option<&Rig>) -> Vec<(String, Placement)> {
    let bare = set.offsets.iter().find(|held| held.row.is_none()).map_or(NO_OFFSET, |held| held.name).to_owned();
    let rows = rig.map_or(0, |rig| rig.model.alignment.len());
    let name = |row: usize| set.offsets.iter().find(|held| held.row == Some(row)).map_or_else(|| format!("Row {row}"), |held| held.name.to_owned());

    std::iter::once((bare, Placement::Bare)).chain((0..rows).map(|row| (name(row), Placement::Row(row)))).collect()
}

#[derive(Clone)]
pub(crate) struct Rigs {
    platform: Platform,
    held: Rc<RefCell<HashMap<String, Option<Arc<Rig>>>>>,
    inflight: Rc<RefCell<HashSet<String>>>,
    tick: Signal<u32>,
}

pub(crate) fn provide_rigs(platform: &Platform) -> Rigs {
    let platform = platform.clone();

    use_context_provider(move || Rigs { platform, held: Rc::default(), inflight: Rc::default(), tick: Signal::new(0) })
}

pub(crate) fn rigs() -> Rigs {
    use_context::<Rigs>()
}

fn file_name(source: &Source) -> Option<Box<str>> {
    source.path.file_name().and_then(|name| name.to_str()).map(Box::from)
}

impl Rigs {
    pub(crate) fn forget_all(&self) {
        self.held.borrow_mut().clear();

        let mut tick = self.tick;

        tick += 1;
    }

    pub(crate) fn settled(&self) -> bool {
        self.inflight.borrow().is_empty()
    }

    pub(crate) fn watch(&self) {
        let _ = (self.tick)();
    }

    fn lookup(&self, rigging: &Rigging) -> Sheet {
        let _ = (self.tick)();

        if let Some(known) = self.held.borrow().get(&rigging.id) {
            return Sheet { rig: known.clone(), deferred: false };
        }

        if !self.inflight.borrow_mut().insert(rigging.id.clone()) {
            return Sheet { rig: None, deferred: true };
        }

        let (Some(png), Some(cut), Some(model)) = (file_name(&rigging.png), file_name(&rigging.cut), file_name(&rigging.model)) else {
            self.held.borrow_mut().insert(rigging.id.clone(), None);
            return Sheet { rig: None, deferred: false };
        };
        let pending = self.platform.host().rig(png, cut, model);
        let rigs = self.clone();
        let mut tick = self.tick;
        let id = rigging.id.clone();

        spawn_forever(async move {
            let rig = match pending.await {
                Ok(Some(parts)) => Model::parse(&parts.model).inspect_err(|err| warn!("The model of {id} could not be parsed: {err}")).ok().map(|model| Arc::new(Rig { model, sheet: parts.sheet })),
                Ok(None) => None,
                Err(fault) => {
                    warn!("The rig {id} could not be prepared: {fault}");
                    None
                }
            };

            rigs.held.borrow_mut().insert(id.clone(), rig);
            rigs.inflight.borrow_mut().remove(&id);
            tick += 1;
        });

        Sheet { rig: None, deferred: true }
    }
}

fn load_rig(rigs: &Rigs, set: &MotionSet, id: Option<&str>) -> Sheet {
    let Some(rigging) = id.and_then(|id| set.motions.iter().find(|motion| motion.rig.id == id)).map(|motion| &motion.rig) else {
        return Sheet { rig: None, deferred: false };
    };

    rigs.lookup(rigging)
}

fn load_anim(set: &MotionSet, index: Option<usize>) -> Option<Arc<Animation>> {
    let motion = index.and_then(|index| set.motions.get(index))?;
    let bytes = motion.file.as_ref()?.read().ok()?;

    Animation::parse(&bytes).inspect_err(|err| warn!("An animation of {} could not be parsed: {err}", motion.rig.id)).ok().map(Arc::new)
}

fn to_world(state: ViewerState, point: (f32, f32)) -> (f32, f32) {
    let (width, height) = *state.size.peek();
    let zoom = *state.zoom.peek();
    let pan = *state.pan.peek();

    ((point.0 - width / 2.0) / zoom - pan.0, (point.1 - height / 2.0) / zoom - pan.1)
}

fn finish_selection(mut state: ViewerState) {
    let held = *state.selection.peek();

    state.selection.set(None);

    let Some((start, end)) = held else {
        state.selecting.set(false);
        return;
    };
    let area = (end.0 - start.0).abs() * (end.1 - start.1).abs();

    state.selecting.set(false);

    if area <= MIN_SELECTION_AREA {
        return;
    }

    let a = to_world(state, start);
    let b = to_world(state, end);

    state.region.set(Some(Region { x: a.0.min(b.0), y: a.1.min(b.1), w: (b.0 - a.0).abs(), h: (b.1 - a.1).abs() }));
}

fn cancel_selection(mut state: ViewerState) {
    state.selection.set(None);
    state.selecting.set(false);
}

fn holding(state: ViewerState) -> Option<(i8, f64)> {
    *state.hold.peek()
}

fn zoom_by(mut state: ViewerState, pixels: f32) {
    state.zoom_pending.with_mut(|pending| *pending += pixels);
}

fn settle_zoom(mut state: ViewerState, delta_ms: f64) {
    let pending = *state.zoom_pending.peek();

    if pending == 0.0 {
        return;
    }

    let step = if pending.abs() < ZOOM_EPSILON { pending } else { pending * (1.0 - (-ZOOM_DECAY_RATE * (delta_ms / 1000.0) as f32).exp()) };
    let factor = (step * ZOOM_RATE_PER_PIXEL).exp();

    state.zoom_pending.set(pending - step);
    state.zoom.with_mut(|zoom| *zoom = (*zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX));
}

fn frame_bounds(state: ViewerState, active: Option<&Active>) -> (f32, Option<f32>) {
    if active.is_some_and(|active| active.model) {
        return (0.0, Some(0.0));
    }

    let min = state.loop_start.peek().unwrap_or(0.0).max(0.0);
    let max = state.loop_end.peek().or_else(|| active.and_then(Active::loop_bound).map(|bound| bound.max(0) as f32)).map(|max| max.max(min));

    (min, max)
}

fn clamp_frame(mut state: ViewerState) {
    let active = state.active.peek().clone();
    let (min, max) = frame_bounds(state, active.as_ref());
    let current = *state.frame.peek();
    let clamped = max.map_or(current.max(min), |max| current.clamp(min, max));

    if clamped != current {
        state.frame.set(clamped);
    }
}

fn step(mut state: ViewerState, direction: f32) {
    let active = state.active.peek().clone();
    let (min, max) = frame_bounds(state, active.as_ref());
    let stepped = *state.frame.peek() + direction;
    let next = if stepped < min {
        max.unwrap_or(min)
    } else if max.is_some_and(|max| stepped > max) {
        min
    } else {
        stepped
    };

    state.frame.set(next);
}

#[component]
pub(crate) fn CatViewer(cat: SelectedCat, globals: Shared) -> Element {
    let form = (cat::state().form)();
    let library = use_context::<Library>();
    let generation = library.generation();
    let key = format!("{}:{generation}", cat_animation::set_id(&cat.0, form));
    let motions = use_memo(use_reactive!(|(cat, form, key, globals)| Motions { key, set: Rc::new(cat_animation::motions(&cat.0, form, &globals.0.vault().vfs)) }));

    rsx! {
        Viewer { motions }
    }
}

#[component]
pub(crate) fn EnemyViewer(enemy: SelectedEnemy, globals: Shared) -> Element {
    let library = use_context::<Library>();
    let generation = library.generation();
    let key = format!("{}:{generation}", enemy_animation::set_id(&enemy.0));
    let motions = use_memo(use_reactive!(|(enemy, key, globals)| Motions { key, set: Rc::new(enemy_animation::motions(&enemy.0, &globals.0.vault().vfs)) }));

    rsx! {
        Viewer { motions }
    }
}

#[component]
pub(crate) fn DebugToggles() -> Element {
    let platform = use_context::<Platform>();
    let settings = context::settings();
    let held = settings.read().animation.clone();
    let apply = |platform: Platform, change: fn(&mut Settings, bool)| {
        move |value: bool| {
            let mut settings = settings;

            change(&mut settings.write(), value);
            remember_settings(&platform, &settings.read());
        }
    };
    let (rig, origin, world) = (platform.clone(), platform.clone(), platform.clone());

    rsx! {
        div { class: "debug-toggles",
            Toggle { value: held.show_rig, label: "Show Rig", onchange: apply(rig, |next, value| next.animation.show_rig = value) }
            Toggle { value: held.show_origin, label: "Show Origin", onchange: apply(origin, |next, value| next.animation.show_origin = value) }
            Toggle { value: held.show_world, label: "Show World", onchange: apply(world, |next, value| next.animation.show_world = value) }
        }
    }
}

#[component]
fn Viewer(motions: Memo<Motions>) -> Element {
    let platform = use_context::<Platform>();
    let settings = context::settings();
    let theme = use_context::<Signal<AppTheme>>();
    let mut state = state();
    let live = try_use_context::<Live>().is_none_or(|live| live.0);
    let mut surface: Signal<Option<Rc<dyn Surface>>> = use_signal(|| None);
    let slots = use_memo(move || place(&motions.read().set));
    let index = use_memo(move || {
        let motions = motions.read();
        let label = (state.selected_label)();

        selected_index(&motions.set, &slots.read(), label.as_deref())
    });
    let rig_id = use_memo(move || {
        let motions = motions.read();

        index().and_then(|index| motions.set.motions.get(index)).map(|motion| motion.rig.id.clone())
    });
    let rigs = rigs();
    let sheet = use_memo(move || {
        let motions = motions.read();
        let id = rig_id();

        Parsed(Rc::new(load_rig(&rigs, &motions.set, id.as_deref())))
    });
    let loaded = use_memo(move || {
        let motions = motions.read();
        let sheet = sheet();
        let anim = load_anim(&motions.set, index());

        Held(Rc::new(Loaded { rig: sheet.0.rig.clone(), anim, failed: !sheet.0.deferred && sheet.0.rig.is_none() }))
    });
    let mut mounted = use_signal(|| false);

    use_effect(move || {
        let held = loaded();
        let motions = motions.read();

        if !live {
            return;
        }

        let label = state.selected_label.peek().clone();
        let index = selected_index(&motions.set, &slots.peek(), label.as_deref());
        let motion = index.and_then(|index| motions.set.motions.get(index));
        let active = motion.map(|motion| Active { model: motion.file.is_none(), looping: motion.looping, anim: held.0.anim.clone() });

        state.active.set(active);

        let previous = state.last_set.peek().clone();

        if previous.as_deref() != Some(motions.set.name.as_str()) {
            if previous.as_deref().is_none_or(|last| unit_of(last) != unit_of(&motions.set.name)) {
                state.frame.set(0.0);
            }

            state.last_set.set(Some(motions.set.name.clone()));
        }

        clamp_frame(state);
    });

    let runner = platform.clone();

    use_hook(move || {
        if live {
            spawn(async move { run(runner, settings, theme, state, surface, loaded).await });
        }
    });

    let failed = loaded.read().0.failed;
    let scale = platform.host().pixel_ratio();
    let (width, height) = (state.size)();
    let (pixel_width, pixel_height) = ((width * scale).round().max(1.0) as u32, (height * scale).round().max(1.0) as u32);
    let opener = platform.clone();
    let selecting = (state.selecting)();
    let viewer_class = if selecting { "viewer selecting" } else { "viewer" };
    let serial = use_hook(next_viewer);
    let canvas_id = format!("viewer-canvas-{serial}");
    let overlay_id = format!("viewer-overlay-{serial}");
    let ids = (canvas_id.clone(), overlay_id.clone());

    rsx! {
        if selecting {
            div { class: "select-scrim", onpointerdown: move |_| cancel_selection(state) }
        }
        div {
            class: "{viewer_class}",
            onresize: move |event| {
                if let Ok(size) = event.data().get_content_box_size() {
                    state.size.set((size.width as f32, size.height as f32));
                }
            },
            if failed {
                div { class: "viewer-missing", "{MISSING_FILES_NOTICE}" }
            } else {
                canvas {
                    id: "{canvas_id}",
                    class: "viewer-canvas",
                    width: "{pixel_width}",
                    height: "{pixel_height}",
                    onmounted: move |_| {
                        if mounted() || !live {
                            return;
                        }

                        mounted.set(true);

                        let platform = opener.clone();
                        let (canvas, overlay) = ids.clone();

                        spawn(async move {
                            match platform.host().surface(&canvas, &overlay).await {
                                Ok(opened) => surface.set(Some(opened)),
                                Err(fault) => warn!("The animation canvas could not be opened: {fault}"),
                            }
                        });
                    },
                }
                canvas {
                    id: "{overlay_id}",
                    class: "viewer-overlay",
                    width: "{pixel_width}",
                    height: "{pixel_height}",
                    onpointerdown: move |event| {
                        let point = event.data().element_coordinates();
                        let at = (point.x as f32, point.y as f32);

                        if *state.selecting.peek() {
                            state.selection.set(Some((at, at)));
                        } else {
                            state.drag.set(Some(at));
                        }
                    },
                    onpointermove: move |event| {
                        let point = event.data().element_coordinates();
                        let (x, y) = (point.x as f32, point.y as f32);

                        if *state.selecting.peek() {
                            let started = state.selection.peek().as_ref().map(|(start, _)| *start);

                            if let Some(start) = started {
                                state.selection.set(Some((start, (x, y))));
                            }

                            return;
                        }

                        let Some((last_x, last_y)) = *state.drag.peek() else {
                            return;
                        };
                        let zoom = *state.zoom.peek();

                        state.pan.with_mut(|pan| {
                            pan.0 += (x - last_x) / zoom;
                            pan.1 += (y - last_y) / zoom;
                        });
                        state.drag.set(Some((x, y)));
                    },
                    onpointerup: move |_| {
                        state.drag.set(None);

                        if *state.selecting.peek() {
                            finish_selection(state);
                        }
                    },
                    onpointercancel: move |_| {
                        state.drag.set(None);

                        if *state.selecting.peek() {
                            cancel_selection(state);
                        }
                    },
                    onpointerleave: move |_| state.drag.set(None),
                    oncontextmenu: move |event| event.prevent_default(),
                    onwheel: move |event| {
                        event.prevent_default();

                        let pixels = match event.data().delta() {
                            WheelDelta::Pixels(vector) => vector.y as f32 / WHEEL_NOTCH_PIXELS * LINE_PIXELS,
                            WheelDelta::Lines(vector) => vector.y as f32 / WHEEL_NOTCH_LINES * LINE_PIXELS,
                            WheelDelta::Pages(vector) => vector.y as f32 * LINE_PIXELS * PAGE_LINES,
                        };

                        if pixels != 0.0 {
                            zoom_by(state, -pixels * ZOOM_SCROLL_STRENGTH);
                        }
                    },
                }
            }
            if selecting {
                div { class: "viewer-hint", div { class: "toast hint-banner", "{SELECT_HINT}" } }
            }
            Controls { motions, loaded, slots, surface }
        }
    }
}

struct Reach {
    rig: usize,
    anim: usize,
    offset: Option<usize>,
    bounds: Option<BoundingBox>,
}

type Stamp = (usize, usize, i32, (f32, f32), f32, (u32, u32), bool, bool, bool, bool, Option<Drag>, Option<Region>);

async fn run(
    platform: Platform,
    settings: Signal<Settings>,
    theme: Signal<AppTheme>,
    mut state: ViewerState,
    surface: Signal<Option<Rc<dyn Surface>>>,
    loaded: Memo<Held>,
) {
    let mut last_time: Option<f64> = None;
    let mut last_drawn: Option<Stamp> = None;
    let mut sized: Option<(u32, u32)> = None;
    let mut reach: Option<Reach> = None;
    let mut shown_rig = 0usize;

    loop {
        let now = platform.host().frame().await;
        let delta = last_time.map_or(TICK_MS, |last| (now - last).clamp(0.0, LONGEST_TICK_MS));

        last_time = Some(now);

        let Some(surface) = surface.peek().clone() else {
            continue;
        };

        settle_zoom(state, delta);

        let held = loaded.peek().clone();
        let active = state.active.peek().clone();
        let rig = held.0.rig.as_deref();
        let anim = held.0.anim.as_deref();

        if *state.playing.peek() && rig.is_some() {
            let advance = FRAME_ADVANCE_PER_TICK * (delta / TICK_MS) as f32;
            let current = *state.frame.peek() + advance;
            let start = state.loop_start.peek().unwrap_or(0.0);
            let end = state.loop_end.peek().or_else(|| active.as_ref().and_then(Active::loop_bound).map(|bound| bound.max(0) as f32));
            let next = match end {
                Some(end) if current > end => start,
                _ => current,
            };

            state.frame.set(next);
        } else if let Some((direction, held_ms)) = holding(state) {
            let total = held_ms + delta;

            state.hold.set(Some((direction, total)));

            if total > HOLD_DELAY_MS {
                step(state, direction as f32 * HOLD_FRAMES_PER_SEC * (delta / 1000.0) as f32);
            }
        }

        let (width, height) = *state.size.peek();
        let scale = platform.host().pixel_ratio();
        let pixels = ((width * scale).round().max(1.0) as u32, (height * scale).round().max(1.0) as u32);

        if sized != Some(pixels) {
            surface.resize(pixels.0, pixels.1, scale);
            sized = Some(pixels);
        }

        let frame = active.as_ref().map_or(*state.frame.peek(), |active| active.playback_frame(*state.frame.peek())).floor() as i32;
        let pan = *state.pan.peek();
        let zoom = *state.zoom.peek();
        let show = settings.peek().animation.clone();
        let rig_key = held.0.rig.as_ref().map_or(0, |rig| Arc::as_ptr(rig) as usize);
        let anim_key = held.0.anim.as_ref().map_or(0, |anim| Arc::as_ptr(anim) as usize);
        let offset = offset_of(rig, *state.placement.peek());
        let selecting = *state.selecting.peek();
        let selection = *state.selection.peek();
        let region = *state.region.peek();
        let stamp: Stamp = (rig_key, anim_key, frame, pan, zoom, pixels, show.show_rig, show.show_origin, show.show_world, selecting, selection, region);

        if last_drawn == Some(stamp) && reach.as_ref().is_none_or(|held| held.offset == offset) {
            continue;
        }

        last_drawn = Some(stamp);

        let palette = theme.peek().palette();
        let [r, g, b] = palette.background.channels();
        let clear = [r, g, b, 1.0];
        let view = View { size: (width, height), pan, zoom };
        let camera = overlay::camera(&view, selecting, selection, region);
        let complete = anim.is_some() || active.as_ref().is_some_and(|active| active.model);
        let rig = match rig {
            Some(rig) if complete => rig,
            Some(_) if rig_key == shown_rig => continue,
            _ => {
                shown_rig = 0;
                surface.draw(&Scene { atlas: None, parts: &[], shapes: &camera, pan, zoom, clear });
                continue;
            }
        };

        shown_rig = rig_key;

        let frames: Vec<FrameData> = resolve_frame(rig, anim, frame, offset);
        let parts: Vec<Painted> = frames.iter().map(|frame| Painted { frame: frame.clone(), tint: NO_TINT }).collect();
        let bounds = show.show_world.then(|| {
            let stale = reach.as_ref().is_none_or(|held| held.rig != rig_key || held.anim != anim_key || held.offset != offset);

            if stale {
                let anims: Vec<&Animation> = anim.into_iter().collect();
                let bounds = rig.calculate_bounds(&anims, show.bounds_cull as f32 / CULL_SCALE, None, offset);

                reach = Some(Reach { rig: rig_key, anim: anim_key, offset, bounds });
            }

            reach.as_ref().and_then(|held| held.bounds)
        });
        let pose = Pose { rig, anim, frame, offset, frames: &frames };
        let mut shapes = overlay::shapes(&pose, bounds.flatten(), &view, &show);

        shapes.extend(camera);
        let atlas = rig.sheet.image_data.as_ref().map(|image| Atlas { key: Arc::as_ptr(image) as usize, width: image.width(), height: image.height(), pixels: image.as_raw() });

        surface.draw(&Scene { atlas, parts: &parts, shapes: &shapes, pan, zoom, clear });
    }
}
