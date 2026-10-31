use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use tracing::{debug, info, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use omote::host::{Backend, Clip, Clock, Equipment, Files, Phase, Playing, Session};
use omote::platform::battle::{AudioCommand, BattleCommand, BattleEvent, OptionsWire, SetupWire, SheetWire, Standing};

use crate::protocol::{Answer, HELLO, Reply};
use crate::worker::{self, ChunkLoader, Engine};

const TICK_MS: i32 = 16;
const FADE_HOLD: f64 = 0.0;

type Ticker = Closure<dyn FnMut()>;

struct WorkerFiles {
    loader: Arc<ChunkLoader>,
}

impl Files for WorkerFiles {
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        kokoro::Loader::load(self.loader.as_ref(), name).map(|bytes| bytes.to_vec())
    }

    fn contains(&self, name: &str) -> bool {
        self.loader.has(name)
    }

    fn names(&self) -> Vec<Box<str>> {
        self.loader.names()
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
    ticker: Option<Ticker>,
    interval: Option<i32>,
}

thread_local! {
    static ARENA: RefCell<Option<Arena>> = const { RefCell::new(None) };
    static FONTS: RefCell<Vec<Arc<[u8]>>> = const { RefCell::new(Vec::new()) };
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

struct Launch {
    setup: Box<SetupWire>,
    options: OptionsWire,
    phone: bool,
    width: f32,
    height: f32,
    prefixes: Vec<Box<str>>,
    pools: Vec<Box<str>>,
    priority: Vec<String>,
}

pub(crate) fn command(engine: &Rc<Engine>, command: BattleCommand) {
    match command {
        BattleCommand::Prepare { fonts } => spawn_local(prepare(fonts)),
        BattleCommand::Start { setup, options, phone, width, height, prefixes, pools, priority } => start(engine, Launch { setup, options, phone, width, height, prefixes, pools, priority }),
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

fn with(apply: impl FnOnce(&mut Arena)) {
    ARENA.with(|slot| {
        if let Some(arena) = slot.borrow_mut().as_mut() {
            apply(arena);
        }
    });
}

fn start(engine: &Rc<Engine>, launch: Launch) {
    let Launch { setup, options, phone, width, height, prefixes, pools, priority } = launch;
    let fonts = FONTS.with(|fonts| fonts.borrow().clone());

    if fonts.is_empty() {
        warn!("The battle is starting without its fonts; labels will be blank");
    }

    let fresh = ARENA.with(|slot| slot.borrow().is_none());

    if fresh {
        let audio = Rc::new(WorkerAudio::default());
        let equipment = Equipment {
            files: Rc::new(WorkerFiles { loader: Arc::clone(&engine.loader) }),
            backend: Rc::new(AudioLink(Rc::clone(&audio))),
            clock: Rc::new(WorkerClock),
            fonts,
            priority: priority.clone(),
        };
        let session = Session::new(equipment);

        ARENA.with(|slot| *slot.borrow_mut() = Some(Arena { session, audio, sent: HashSet::new(), standing: None, diagnosing: false, forgetting: false, warming: None, ticker: None, interval: None }));
    }

    let frames = engine.loader.warmable(&prefixes, &pools);

    with(|arena| {
        arena.session.relocalize(priority);
        arena.session.equip((*setup).into());
        arena.session.set_phone(phone);
        arena.session.resize(width, height);
        arena.session.configure(options.into());
        arena.session.expect_warmth();
        arena.session.begin();
        arena.warming = Some(frames);
    });

    schedule(engine);
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

    if idle {
        with(|arena| {
            if std::mem::take(&mut arena.forgetting) {
                arena.session.forget_assets();
                arena.sent.clear();
            }
        });
    }
}
