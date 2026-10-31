mod film;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use tracing::{debug, info, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use kokoro::domains::sandbox::replay::{self as tape, Footage, Opened};
use kokoro::domains::settings::ReplaySource;
use omote::host::{Backend, Clip, Clock, Equipment, Files, Label, Phase, Playing, Session, VERSION};
use omote::platform::battle::{AudioCommand, BattleCommand, BattleEvent, Launch, Recorded, ReelWire, SheetWire, Standing};

use crate::protocol::{Answer, HELLO, Reply};
use crate::worker::{self, ChunkLoader, Engine};

const TICK_MS: i32 = 16;
const FADE_HOLD: f64 = 0.0;
const UNKNOWN_VERSION: &str = "Unknown";
const REPLAY_POOLS: [&str; 2] = ["caf", "roster:battle"];
const FETCH_SHARE: f32 = 0.1;
const UNPACK_SHARE: f32 = 0.9;
const PROGRESS_STEP: f32 = 0.01;

type Ticker = Closure<dyn FnMut()>;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Fallback {
    #[default]
    Whole,
    Units,
    Nothing,
}

#[derive(Default)]
struct Overlay {
    files: HashMap<Box<str>, Arc<[u8]>>,
    pinned: HashMap<Box<str>, Box<str>>,
    fallback: Fallback,
}

impl Overlay {
    fn active(&self) -> bool {
        self.fallback != Fallback::Whole || !self.files.is_empty() || !self.pinned.is_empty()
    }

    fn falls_back(&self, name: &str) -> bool {
        match self.fallback {
            Fallback::Whole => true,
            Fallback::Units => tape::unit_file(name).is_some(),
            Fallback::Nothing => false,
        }
    }
}

type Vault = Rc<RefCell<Overlay>>;
type Carried = Rc<HashMap<Box<str>, Arc<[u8]>>>;

struct Reel {
    footage: Footage,
    files: Option<Carried>,
    pinned: bool,
}

#[derive(Clone)]
struct Bundled {
    name: Rc<str>,
    bytes: Rc<[u8]>,
    files: Carried,
}

#[derive(Clone)]
struct Kept {
    footage: Rc<Footage>,
    bundle: Option<Bundled>,
    carried: Option<Carried>,
}

impl Kept {
    fn opened(opened: Opened, name: Rc<str>, bytes: Rc<[u8]>) -> Self {
        let files = Rc::new(opened.by_request());

        Self { footage: Rc::new(opened.footage), bundle: Some(Bundled { name, bytes, files }), carried: None }
    }

    fn recorded(footage: Footage, read: Vec<(Box<str>, Arc<[u8]>)>) -> Self {
        let by_stored: HashMap<Box<str>, Arc<[u8]>> = read.into_iter().collect();
        let mut carried = HashMap::new();

        for (requested, stored) in &footage.manifest {
            if let Some(bytes) = by_stored.get(stored) {
                carried.insert(requested.clone(), Arc::clone(bytes));
                carried.insert(stored.clone(), Arc::clone(bytes));
            }
        }

        Self { footage: Rc::new(footage), bundle: None, carried: (!carried.is_empty()).then(|| Rc::new(carried)) }
    }

    fn reel(&self) -> Reel {
        match &self.bundle {
            Some(bundle) => Reel { footage: self.footage.as_ref().clone(), files: Some(Rc::clone(&bundle.files)), pinned: false },
            None => Reel { footage: self.footage.as_ref().clone(), files: self.carried.clone(), pinned: true },
        }
    }

    fn read(&self, engine: &Engine, name: &str) -> Option<Arc<[u8]>> {
        self.carried.as_ref().and_then(|carried| carried.get(name).cloned()).or_else(|| engine.loader.vanilla(name))
    }
}

fn pins(footage: &Footage, files: Option<&Carried>) -> HashMap<Box<str>, Box<str>> {
    let mut pinned = HashMap::new();

    for (requested, stored) in &footage.manifest {
        for name in [requested, stored] {
            if files.is_none_or(|files| !files.contains_key(name)) {
                pinned.insert(name.clone(), stored.clone());
            }
        }
    }

    pinned
}

struct WorkerFiles {
    loader: Arc<ChunkLoader>,
    overlay: Vault,
}

impl Files for WorkerFiles {
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let pinned = {
            let overlay = self.overlay.borrow();

            if let Some(bytes) = overlay.files.get(name) {
                return Some(bytes.to_vec());
            }

            match overlay.pinned.get(name) {
                Some(stored) => Some(stored.clone()),
                None if overlay.falls_back(name) => None,
                None => return None,
            }
        };

        match pinned {
            Some(stored) => self.loader.vanilla(&stored),
            None => kokoro::Loader::load(self.loader.as_ref(), name),
        }
        .map(|bytes| bytes.to_vec())
    }

    fn contains(&self, name: &str) -> bool {
        let overlay = self.overlay.borrow();

        if overlay.files.contains_key(name) {
            return true;
        }

        match overlay.pinned.get(name) {
            Some(stored) => self.loader.has_vanilla(stored),
            None => overlay.falls_back(name) && self.loader.has(name),
        }
    }

    fn names(&self) -> Vec<Box<str>> {
        let overlay = self.overlay.borrow();
        let mut names: BTreeSet<Box<str>> = match overlay.fallback {
            Fallback::Whole => self.loader.names().into_iter().collect(),
            Fallback::Units => self.loader.names().into_iter().filter(|name| overlay.falls_back(name)).collect(),
            Fallback::Nothing => BTreeSet::new(),
        };

        names.extend(overlay.files.keys().cloned());
        names.extend(overlay.pinned.iter().filter(|(_, stored)| self.loader.has_vanilla(stored)).map(|(name, _)| name.clone()));
        names.into_iter().collect()
    }
}

struct WorkerClock;

impl Clock for WorkerClock {
    fn now_ms(&self) -> f64 {
        worker::now()
    }

    fn epoch_micros(&self) -> i64 {
        (js_sys::Date::now() * 1000.0) as i64
    }
}

#[derive(Default)]
struct WorkerAudio {
    commands: RefCell<Vec<AudioCommand>>,
    loaded: RefCell<HashSet<Box<str>>>,
    next: Cell<u32>,
}

struct WorkerVoice {
    id: u32,
    audio: Rc<WorkerAudio>,
    until: Option<f64>,
    stopped: Cell<bool>,
}

impl Playing for WorkerVoice {
    fn set_level(&self, level: f32) {
        self.audio.commands.borrow_mut().push(AudioCommand::Level { voice: self.id, level });
    }

    fn stop(&self) {
        if !self.stopped.replace(true) {
            self.audio.commands.borrow_mut().push(AudioCommand::Stop { voice: self.id });
        }
    }

    fn busy(&self) -> bool {
        !self.stopped.get() && self.until.is_none_or(|until| worker::now() < until)
    }
}

struct AudioLink(Rc<WorkerAudio>);

impl Backend for AudioLink {
    fn start(&self, clip: &Clip, looped: bool, level: f32) -> Option<Box<dyn Playing>> {
        let (name, until) = match clip {
            Clip::Effect { name, effect } => {
                if self.0.loaded.borrow_mut().insert(Box::from(name.as_str())) {
                    self.0.commands.borrow_mut().push(AudioCommand::Effect {
                        clip: Box::from(name.as_str()),
                        channels: effect.channels,
                        rate: effect.rate,
                        samples: effect.samples.to_vec(),
                    });
                }

                let frames = effect.samples.len() as f64 / f64::from(effect.channels.max(1));
                let seconds = frames / f64::from(effect.rate.max(1));

                (name.as_str(), (!looped).then(|| worker::now() + seconds * 1000.0 + FADE_HOLD))
            }
            Clip::Track { name, bytes } => {
                if self.0.loaded.borrow_mut().insert(Box::from(name.as_str())) {
                    self.0.commands.borrow_mut().push(AudioCommand::Track { clip: Box::from(name.as_str()), bytes: bytes.to_vec() });
                }

                (name.as_str(), None)
            }
        };
        let id = self.0.next.get().wrapping_add(1);

        self.0.next.set(id);
        self.0.commands.borrow_mut().push(AudioCommand::Start { voice: id, clip: Box::from(name), looped, level });

        Some(Box::new(WorkerVoice { id, audio: Rc::clone(&self.0), until, stopped: Cell::new(false) }))
    }
}

struct Arena {
    session: Session,
    audio: Rc<WorkerAudio>,
    sent: HashSet<u64>,
    standing: Option<Standing>,
    diagnosing: bool,
    forgetting: bool,
    warming: Option<Vec<usize>>,
    priority: Vec<String>,
    overlay: Vault,
    ticker: Option<Ticker>,
    interval: Option<i32>,
}

thread_local! {
    static ARENA: RefCell<Option<Arena>> = const { RefCell::new(None) };
    static FONTS: RefCell<Vec<Arc<[u8]>>> = const { RefCell::new(Vec::new()) };
    static LATEST: RefCell<Option<Option<Kept>>> = const { RefCell::new(None) };
    static HELD: RefCell<Option<Reel>> = const { RefCell::new(None) };
}

fn post(event: BattleEvent) {
    worker::post(Answer { id: HELLO, reply: Reply::Battle(event) });
}

pub(crate) fn fonts_ready() -> bool {
    FONTS.with(|fonts| !fonts.borrow().is_empty())
}

pub(crate) async fn prepare(urls: Vec<Box<str>>) {
    if fonts_ready() {
        return;
    }

    let Some(scope) = worker::scope() else {
        return;
    };
    let mut fonts = Vec::with_capacity(urls.len());

    for url in &urls {
        match worker::fetch_bytes(&scope, url, None, false).await {
            Ok(bytes) => fonts.push(Arc::from(bytes)),
            Err(reason) => warn!("A battle font could not be fetched from {url}: {reason}"),
        }
    }

    info!(faces = fonts.len(), "Battle fonts loaded");
    FONTS.with(|held| *held.borrow_mut() = fonts);
}

pub(crate) fn command(engine: &Rc<Engine>, command: BattleCommand) {
    match command {
        BattleCommand::Prepare { fonts } => spawn_local(prepare(fonts)),
        BattleCommand::Start(launch) => start(engine, *launch),
        BattleCommand::Watch { reel, checked, source, phone, width, height, priority } => {
            let engine = Rc::clone(engine);

            spawn_local(async move { watch(&engine, reel, checked, source, phone, (width, height), priority).await });
        }
        BattleCommand::Latest => {
            let engine = Rc::clone(engine);

            spawn_local(async move {
                let latest = latest(&engine).await;

                post(BattleEvent::Latest(latest.map(|kept| Box::new(recorded(&engine, &kept)))));
            });
        }
        BattleCommand::ExportLatest => {
            let engine = Rc::clone(engine);

            spawn_local(async move { post(BattleEvent::Exported(export(&engine).await)) });
        }
        BattleCommand::Upload { name, bytes } => {
            let engine = Rc::clone(engine);

            spawn_local(async move {
                let uploaded = upload(&engine, name, bytes).await;

                if let Err(reason) = &uploaded {
                    warn!("A replay could not be uploaded: {reason}");
                }

                post(BattleEvent::Uploaded(uploaded));
            });
        }
        BattleCommand::Film { source, priority } => film::start(engine, source, priority),
        BattleCommand::Roll(count) => film::roll(count),
        BattleCommand::Unroll => film::unroll(),
        BattleCommand::ExportVideo(_) | BattleCommand::AbortVideo => warn!("A page-side video command reached the data worker"),
        BattleCommand::DeleteLatest => {
            let engine = Rc::clone(engine);

            spawn_local(async move { delete(&engine).await });
        }
        BattleCommand::Stop => with(|arena| arena.session.terminate()),
        BattleCommand::Forget => with(|arena| {
            if arena.session.phase() == Phase::Idle {
                arena.session.forget_assets();
                arena.sent.clear();
            } else {
                arena.forgetting = true;
            }
        }),
        BattleCommand::Resume => with(|arena| arena.session.resume()),
        BattleCommand::Restart => with(|arena| arena.session.restart()),
        BattleCommand::Touches(touches) => with(|arena| arena.session.touches().borrow_mut().extend(touches)),
        BattleCommand::Key(action, pressed) => with(|arena| arena.session.key(action, pressed)),
        BattleCommand::Resize(width, height) => with(|arena| arena.session.resize(width, height)),
        BattleCommand::Phone(phone) => with(|arena| arena.session.set_phone(phone)),
        BattleCommand::Volumes(music, effects) => with(|arena| arena.session.retune(music, effects)),
        BattleCommand::Presented => with(|arena| arena.session.presented()),
        BattleCommand::Diagnose(on) => with(|arena| arena.diagnosing = on),
    }
}

pub(crate) fn remount() {
    with(|arena| {
        arena.session.refit();
        arena.sent.clear();
    });
}

fn with(apply: impl FnOnce(&mut Arena)) {
    ARENA.with(|slot| {
        if let Some(arena) = slot.borrow_mut().as_mut() {
            apply(arena);
        }
    });
}

fn ensure(engine: &Rc<Engine>, priority: Vec<String>) {
    let fresh = ARENA.with(|slot| slot.borrow().is_none());

    if fresh {
        let fonts = FONTS.with(|fonts| fonts.borrow().clone());

        if fonts.is_empty() {
            warn!("The battle is starting without its fonts; labels will be blank");
        }

        let audio = Rc::new(WorkerAudio::default());
        let overlay: Vault = Rc::default();
        let equipment = Equipment {
            files: Rc::new(WorkerFiles { loader: Arc::clone(&engine.loader), overlay: Rc::clone(&overlay) }),
            backend: Rc::new(AudioLink(Rc::clone(&audio))),
            clock: Rc::new(WorkerClock),
            fonts,
            priority: priority.clone(),
        };
        let session = Session::new(equipment);

        ARENA.with(|slot| {
            *slot.borrow_mut() =
                Some(Arena { session, audio, sent: HashSet::new(), standing: None, diagnosing: false, forgetting: false, warming: None, priority, overlay, ticker: None, interval: None })
        });

        return;
    }

    with(|arena| {
        if arena.priority != priority {
            arena.session.relocalize(priority.clone());
            arena.sent.clear();
            arena.priority = priority;
        }
    });
}

fn start(engine: &Rc<Engine>, launch: Launch) {
    let Launch { setup, options, phone, width, height, prefixes, pools, priority, record, app, map, stage } = launch;
    let frames = engine.loader.warmable(&prefixes, &pools);

    ensure(engine, priority);
    with(|arena| {
        arena.session.equip(setup.into());
        arena.session.set_phone(phone);
        arena.session.resize(width, height);
        arena.session.configure(options.into());
        arena.session.set_recording(record);
        arena.session.label(Label { app, map, stage });
        arena.session.expect_warmth();
        arena.session.begin();
        arena.warming = Some(frames);
    });

    schedule(engine);
}

async fn latest(engine: &Rc<Engine>) -> Option<Kept> {
    if let Some(known) = LATEST.with(|latest| latest.borrow().clone()) {
        return known;
    }

    let name: Rc<str> = worker::recall(engine, worker::UPLOADED_NAME).await.and_then(|bytes| String::from_utf8(bytes).ok()).map_or_else(|| Rc::from(""), Rc::from);
    let uploaded = worker::recall(engine, worker::UPLOADED_FILE).await.and_then(|bytes| {
        Footage::open(&bytes)
            .inspect_err(|err| warn!("The uploaded battle could not be read back: {err}"))
            .ok()
            .map(|opened| Kept::opened(opened, name, Rc::from(bytes)))
    });
    let held = match uploaded {
        Some(kept) => Some(kept),
        None => {
            let footage = worker::recall(engine, worker::LATEST_FILE).await.and_then(|bytes| {
                postcard::from_bytes::<Footage>(&bytes).inspect_err(|err| warn!("The latest battle could not be read back: {err}")).ok()
            });
            let read = match footage {
                Some(_) => worker::recall(engine, worker::LATEST_FILES).await.map_or_else(Vec::new, |bytes| {
                    postcard::from_bytes::<Vec<(Box<str>, Vec<u8>)>>(&bytes)
                        .inspect_err(|err| warn!("The mod files the latest battle read could not be read back: {err}"))
                        .unwrap_or_default()
                }),
                None => Vec::new(),
            };

            footage.map(|footage| Kept::recorded(footage, read.into_iter().map(|(name, bytes)| (name, Arc::from(bytes))).collect()))
        }
    };

    LATEST.with(|latest| *latest.borrow_mut() = Some(held.clone()));

    held
}

fn remember(engine: &Rc<Engine>, footage: Footage) {
    let read = engine.loader.modded(footage.stored());
    let stored: Vec<(Box<str>, Vec<u8>)> = read.iter().map(|(name, bytes)| (name.clone(), bytes.to_vec())).collect();
    let kept = Kept::recorded(footage, read);

    LATEST.with(|latest| *latest.borrow_mut() = Some(Some(kept.clone())));
    post(BattleEvent::Latest(Some(Box::new(recorded(engine, &kept)))));

    let engine = Rc::clone(engine);

    spawn_local(async move {
        match postcard::to_allocvec(kept.footage.as_ref()) {
            Ok(bytes) => {
                if let Err(reason) = worker::keep(&engine, worker::LATEST_FILE, &bytes).await {
                    warn!("The latest battle could not be kept in browser storage: {reason}");
                }
            }
            Err(err) => warn!("The latest battle could not be encoded: {err}"),
        }

        let files = match stored.is_empty() {
            true => worker::discard(&engine, worker::LATEST_FILES).await,
            false => match postcard::to_allocvec(&stored) {
                Ok(bytes) => worker::keep(&engine, worker::LATEST_FILES, &bytes).await,
                Err(err) => Err(format!("the files could not be encoded: {err}")),
            },
        };

        if let Err(reason) = files {
            warn!("The mod files the latest battle read could not be kept in browser storage: {reason}");
        }

        for name in [worker::UPLOADED_FILE, worker::UPLOADED_NAME] {
            if let Err(reason) = worker::discard(&engine, name).await {
                warn!("{name} could not be dropped from browser storage: {reason}");
            }
        }
    });
}

fn recorded(engine: &Rc<Engine>, kept: &Kept) -> Recorded {
    let footage = &kept.footage;
    let (bytes, uploaded) = match &kept.bundle {
        Some(bundle) => (bundle.bytes.len() as u64, Some(bundle.name.as_ref().to_owned())),
        None => {
            let carried = |name: &str| kept.carried.as_ref().and_then(|carried| carried.get(name)).map(|bytes| bytes.len());
            let files: usize = footage.stored().into_iter().filter_map(|name| carried(name).or_else(|| engine.loader.vanilla_size(name))).sum();

            ((footage.text_size() + files) as u64, None)
        }
    };

    Recorded { save: footage.save.clone(), frames: footage.frames, assets: footage.stored().len(), bytes, uploaded }
}

async fn upload(engine: &Rc<Engine>, name: String, bytes: Vec<u8>) -> Result<(), String> {
    let shown = Cell::new(0.0f32);

    post(BattleEvent::Uploading(0.0));

    let opened = Footage::open_with(&bytes, |share| {
        let share = UNPACK_SHARE * share;

        if share - shown.get() >= PROGRESS_STEP {
            shown.set(share);
            post(BattleEvent::Uploading(share));
        }
    })?;
    let bytes: Rc<[u8]> = Rc::from(bytes);
    let kept = Kept::opened(opened, Rc::from(name.as_str()), Rc::clone(&bytes));

    post(BattleEvent::Uploading(UNPACK_SHARE));

    let stored = match worker::keep(engine, worker::UPLOADED_FILE, &bytes).await {
        Ok(()) => worker::keep(engine, worker::UPLOADED_NAME, name.as_bytes()).await,
        Err(reason) => Err(reason),
    };

    match stored {
        Ok(()) => {
            for name in [worker::LATEST_FILE, worker::LATEST_FILES] {
                if let Err(reason) = worker::discard(engine, name).await {
                    warn!("{name} could not be dropped from browser storage: {reason}");
                }
            }
        }
        Err(reason) => warn!("The uploaded battle could not be kept in browser storage and lasts only until the page closes: {reason}"),
    }

    info!(bytes = bytes.len(), frames = kept.footage.frames, "A replay was uploaded as the latest battle");
    LATEST.with(|latest| *latest.borrow_mut() = Some(Some(kept.clone())));
    post(BattleEvent::Uploading(1.0));
    post(BattleEvent::Latest(Some(Box::new(recorded(engine, &kept)))));

    Ok(())
}

async fn delete(engine: &Rc<Engine>) {
    LATEST.with(|latest| *latest.borrow_mut() = Some(None));
    post(BattleEvent::Latest(None));

    for name in [worker::LATEST_FILE, worker::LATEST_FILES, worker::UPLOADED_FILE, worker::UPLOADED_NAME] {
        if let Err(reason) = worker::discard(engine, name).await {
            warn!("{name} could not be dropped from browser storage: {reason}");
        }
    }

    info!("The latest battle was deleted");
}

async fn export(engine: &Rc<Engine>) -> Result<Vec<u8>, String> {
    let kept = latest(engine).await.ok_or("there is no latest battle")?;

    if let Some(bundle) = &kept.bundle {
        post(BattleEvent::Exporting(1.0));

        return Ok(bundle.bytes.to_vec());
    }

    let footage = &kept.footage;
    let frames = engine.loader.frames_of(footage.stored());
    let shown = Cell::new(0.0f32);
    let show = |share: f32| {
        if share - shown.get() >= PROGRESS_STEP {
            shown.set(share);
            post(BattleEvent::Exporting(share));
        }
    };

    post(BattleEvent::Exporting(0.0));
    worker::warm_frames(engine, frames, false, |done, total| show(FETCH_SHARE * done as f32 / total.max(1) as f32)).await;

    let (mut packer, missing) = footage.packer(|name| kept.read(engine, name))?;

    if !missing.is_empty() {
        warn!(count = missing.len(), "Files the latest battle read are no longer published and were left out of the replay: {missing:?}");
    }

    while packer.step()? {
        show(FETCH_SHARE + (1.0 - FETCH_SHARE) * packer.progress());
        worker::breathe().await;
    }

    let packed = packer.finish()?;

    post(BattleEvent::Exporting(1.0));
    info!(bytes = packed.len(), "The latest battle was packed");

    Ok(packed)
}

async fn watch(engine: &Rc<Engine>, reel: ReelWire, checked: bool, source: ReplaySource, phone: bool, size: (f32, f32), priority: Vec<String>) {
    let opened = match reel {
        ReelWire::Latest => latest(engine).await.map(|kept| kept.reel()).ok_or_else(|| "there is no latest battle".to_owned()),
        ReelWire::Held => HELD.with(|held| held.borrow_mut().take()).ok_or_else(|| "the replay is no longer held".to_owned()),
    };
    let reel = match opened {
        Ok(reel) => reel,
        Err(reason) => {
            warn!("A replay could not be watched: {reason}");
            post(BattleEvent::Rejected(format!("The replay could not be watched: {reason}")));

            return;
        }
    };

    if !checked && reel.footage.save.version != VERSION {
        let recorded = if reel.footage.save.version.is_empty() { UNKNOWN_VERSION.to_owned() } else { reel.footage.save.version.clone() };

        HELD.with(|held| *held.borrow_mut() = Some(reel));
        post(BattleEvent::Foreign(recorded));

        return;
    }

    let Reel { footage, files, pinned } = reel;
    let (fresh, frames) = layering(engine, &footage, files, pinned, source);

    ensure(engine, priority);
    with(|arena| {
        let changed = arena.overlay.borrow().active() || fresh.active();

        *arena.overlay.borrow_mut() = fresh;

        if changed {
            arena.session.refit();
            arena.sent.clear();
        }

        arena.session.set_phone(phone);
        arena.session.resize(size.0, size.1);
        arena.session.expect_warmth();
        arena.session.watch(footage);
        arena.warming = Some(frames);
    });

    schedule(engine);
}

fn layering(engine: &Engine, footage: &Footage, files: Option<Carried>, pinned: bool, source: ReplaySource) -> (Overlay, Vec<usize>) {
    let own = source != ReplaySource::Vfs && (files.is_some() || pinned);
    let carried = files.filter(|_| own);
    let pinned = if own && pinned { pins(footage, carried.as_ref()) } else { HashMap::new() };
    let fallback = match (source, own) {
        (_, false) => Fallback::Whole,
        (ReplaySource::Bcv, true) => Fallback::Nothing,
        (_, true) => Fallback::Units,
    };
    let mut frames = match fallback {
        Fallback::Whole => {
            let pools: Vec<Box<str>> = REPLAY_POOLS.iter().map(|pool| Box::from(*pool)).collect();
            let mut frames = engine.loader.warmable(&[], &pools);

            frames.extend(engine.loader.frames_of(footage.requested().chain(footage.stored())));
            frames
        }
        Fallback::Units => {
            let names = engine.loader.names();

            engine.loader.frames_of(names.iter().map(|name| &**name).filter(|name| tape::unit_file(name).is_some() && carried.as_ref().is_none_or(|carried| !carried.contains_key(*name))))
        }
        Fallback::Nothing => Vec::new(),
    };

    frames.extend(engine.loader.frames_of(pinned.values().map(|stored| &**stored)));

    frames.sort_unstable();
    frames.dedup();

    (Overlay { files: carried.map(|carried| carried.as_ref().clone()).unwrap_or_default(), pinned, fallback }, frames)
}

fn warm(engine: Rc<Engine>, frames: Vec<usize>) {
    spawn_local(async move {
        let total = frames.len();

        post(BattleEvent::Warming { done: 0, total });
        worker::warm_frames(&engine, frames, false, |done, total| post(BattleEvent::Warming { done, total })).await;
        post(BattleEvent::Warming { done: total, total });
        with(|arena| arena.session.warmed());
    });
}

fn schedule(engine: &Rc<Engine>) {
    let Some(scope) = worker::scope() else {
        return;
    };
    let already = ARENA.with(|slot| slot.borrow().as_ref().is_some_and(|arena| arena.interval.is_some()));

    if already {
        return;
    }

    let engine = Rc::clone(engine);
    let ticker: Ticker = Closure::new(move || tick(&engine));
    let interval = scope.set_interval_with_callback_and_timeout_and_arguments_0(ticker.as_ref().unchecked_ref(), TICK_MS);

    with(|arena| {
        arena.ticker = Some(ticker);
        arena.interval = interval.ok();
    });
}

fn tick(engine: &Rc<Engine>) {
    let mut idle = false;
    let mut pending: Option<Vec<usize>> = None;
    let mut filmed: Option<Footage> = None;

    ARENA.with(|slot| {
        let mut held = slot.borrow_mut();
        let Some(arena) = held.as_mut() else {
            return;
        };
        let drawn = arena.session.tick();
        let standing = Standing { phase: arena.session.phase(), covered: arena.session.covered(), surfaced: arena.session.surfaced() };

        if standing.phase == Phase::Loading {
            pending = arena.warming.take();
        }

        filmed = arena.session.take_footage();

        if arena.session.take_cut() {
            post(BattleEvent::Cut);
        }

        if arena.standing.as_ref() != Some(&standing) {
            if let Some(failure) = arena.session.failure().filter(|_| standing.phase != Phase::Running) {
                post(BattleEvent::Failure(failure.to_owned()));
            }

            arena.standing = Some(standing.clone());
            post(BattleEvent::Standing(standing.clone()));
        }

        if drawn {
            let sheets: Vec<SheetWire> = arena
                .session
                .sheets()
                .borrow()
                .iter()
                .filter(|(_, sheet)| !arena.sent.contains(&sheet.stamp))
                .map(|(name, sheet)| SheetWire { name: name.clone(), width: sheet.width, height: sheet.height, pixels: sheet.pixels.to_vec() })
                .collect();

            if !sheets.is_empty() {
                for sheet in arena.session.sheets().borrow().values() {
                    arena.sent.insert(sheet.stamp);
                }

                debug!(count = sheets.len(), "Battle sheets handed to the page");
                post(BattleEvent::Sheets(sheets));
            }

            post(BattleEvent::Frame(arena.session.frame().borrow().clone()));
        }

        let commands: Vec<AudioCommand> = std::mem::take(&mut *arena.audio.commands.borrow_mut());

        if !commands.is_empty() {
            post(BattleEvent::Audio(commands));
        }

        if let Some(options) = arena.session.take_options() {
            post(BattleEvent::Options(options.into()));
        }

        if arena.diagnosing && let Some(diagnostics) = arena.session.diagnose() {
            post(BattleEvent::Diagnostics(diagnostics));
        }

        idle = standing.phase == Phase::Idle;

        if idle {
            if let (Some(scope), Some(interval)) = (worker::scope(), arena.interval.take()) {
                scope.clear_interval_with_handle(interval);
            }

            arena.ticker = None;
        }
    });

    if let Some(frames) = pending {
        warm(Rc::clone(engine), frames);
    }

    if let Some(footage) = filmed {
        remember(engine, footage);
    }

    if idle {
        with(|arena| {
            if arena.overlay.borrow().active() {
                *arena.overlay.borrow_mut() = Overlay::default();
                arena.session.refit();
                arena.sent.clear();
                info!("The replay's own files were dropped");
            }

            if std::mem::take(&mut arena.forgetting) {
                arena.session.forget_assets();
                arena.sent.clear();
            }
        });
    }
}
