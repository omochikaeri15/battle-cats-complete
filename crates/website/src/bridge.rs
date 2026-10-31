use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::{Asset, AssetOptions, asset, manganis};
use futures_channel::oneshot;
use image::RgbaImage;
use js_sys::{Array, ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array};
use tracing::{error, info, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, BlobPropertyBag, Cache, ErrorEvent, Headers, HtmlAnchorElement, HtmlInputElement, HtmlScriptElement, MessageEvent, RegistrationOptions, Response, ResponseInit, Storage, Url, Worker, WorkerOptions, WorkerType,
};

use codex::graphics::rig::{Opaque, SpriteCut, SpriteSheet};
use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::enemy::scanner::EnemyEntry;
use kokoro::domains::settings::ScannerConfig;
use kokoro::domains::stage::scanner::StageBundle;

use omote::host::Phase as BattlePhase;
use omote::platform::battle::{BattleCommand, BattleEvent, BattleWatch};
use omote::platform::mods::{ModCommand, ModState};
use omote::platform::{LOGGING_KEY, LOGGING_ON};
use omote::{Bytes, Fault, Fetched, Host, Pending, Phase, Picked, Picture, Prepared, Progress, Report, RigParts, Script, Surface, Watch};

use crate::battle::{self, Screen, SharedScreen};
use crate::film;
use crate::protocol::{self, Answer, Call, HELLO, PictureKind, Reply, Request, RigWire};
use crate::{viewer, worker};

struct Waiting {
    reply: oneshot::Sender<Reply>,
    report: Option<Report>,
}

type Calls = Rc<RefCell<HashMap<u32, Waiting>>>;

struct Link {
    worker: Worker,
    ready: Cell<bool>,
    outbox: RefCell<Vec<ArrayBuffer>>,
    broken: RefCell<Option<String>>,
    watch: RefCell<Option<Watch>>,
    phase: RefCell<Option<Phase>>,
    screen: SharedScreen,
    battle_watch: RefCell<Option<BattleWatch>>,
    standing: Cell<BattlePhase>,
    next: Cell<u32>,
    painting: Cell<bool>,
}

impl Link {
    fn post(&self, buffer: ArrayBuffer) -> Result<(), Fault> {
        if let Some(reason) = self.broken.borrow().as_ref() {
            return Err(Fault(reason.clone()));
        }

        if !self.ready.get() {
            self.outbox.borrow_mut().push(buffer);
            return Ok(());
        }

        self.worker.post_message_with_transfer(&buffer, &Array::of1(&buffer)).map_err(|err| Fault(format!("the request could not be posted: {err:?}")))
    }

    fn flush(&self) {
        self.ready.set(true);

        for buffer in self.outbox.borrow_mut().drain(..) {
            if let Err(err) = self.worker.post_message_with_transfer(&buffer, &Array::of1(&buffer)) {
                error!("A queued request could not be posted to the data worker: {:?}", err);
            }
        }
    }
}

type FullscreenWatch = Rc<RefCell<Option<Box<dyn Fn(bool)>>>>;

pub(crate) struct WebHost {
    link: Option<Rc<Link>>,
    calls: Calls,
    listeners: Vec<Closure<dyn FnMut(JsValue)>>,
    fullscreen: FullscreenWatch,
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn fullscreen_now() -> bool {
    document().and_then(|document| document.fullscreen_element()).is_some()
}

impl WebHost {
    pub(crate) fn spawn() -> Self {
        let calls: Calls = Rc::new(RefCell::new(HashMap::new()));
        let mut host = Self { link: None, calls: Rc::clone(&calls), listeners: Vec::new(), fullscreen: Rc::default() };
        let on_fullscreen = {
            let watch = Rc::clone(&host.fullscreen);

            Closure::<dyn FnMut(JsValue)>::new(move |_: JsValue| {
                if let Some(watch) = watch.borrow().as_ref() {
                    watch(fullscreen_now());
                }
            })
        };

        if let Some(document) = document()
            && let Err(err) = document.add_event_listener_with_callback(FULLSCREEN_CHANGE, on_fullscreen.as_ref().unchecked_ref())
        {
            warn!("The fullscreen toggle cannot follow the browser: {:?}", err);
        }

        host.listeners.push(on_fullscreen);

        register_worker();

        if let Some(fonts) = web_sys::window().and_then(|window| window.document()).map(|document| document.fonts()) {
            let loading = Closure::<dyn FnMut(JsValue)>::new(|_| info!("Fonts started loading"));
            let done = Closure::<dyn FnMut(JsValue)>::new(|_| info!("Fonts finished loading"));

            fonts.set_onloading(Some(loading.as_ref().unchecked_ref()));
            fonts.set_onloadingdone(Some(done.as_ref().unchecked_ref()));
            host.listeners.push(loading);
            host.listeners.push(done);
        }

        match start() {
            Ok(worker) => {
                let link = Rc::new(Link {
                    worker,
                    ready: Cell::new(false),
                    outbox: RefCell::new(Vec::new()),
                    broken: RefCell::new(None),
                    watch: RefCell::new(None),
                    phase: RefCell::new(None),
                    screen: Rc::new(RefCell::new(Screen::default())),
                    battle_watch: RefCell::new(None),
                    standing: Cell::new(BattlePhase::Idle),
                    next: Cell::new(HELLO + 1),
                    painting: Cell::new(false),
                });
                let on_message = {
                    let (calls, link) = (Rc::clone(&calls), Rc::clone(&link));

                    Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                        if let Ok(event) = event.dyn_into::<MessageEvent>() {
                            received(&calls, &link, event);
                        }
                    })
                };
                let on_error = {
                    let (calls, link) = (Rc::clone(&calls), Rc::clone(&link));

                    Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| failed(&calls, &link, event))
                };

                let on_leave = {
                    let worker = link.worker.clone();

                    Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                        let kept = Reflect::get(&event, &JsValue::from_str(PERSISTED)).ok().and_then(|value| value.as_bool()).unwrap_or(false);

                        if !kept {
                            worker.terminate();
                        }
                    })
                };

                link.worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                link.worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));

                if let Some(window) = web_sys::window()
                    && let Err(err) = window.add_event_listener_with_callback(PAGE_HIDE, on_leave.as_ref().unchecked_ref())
                {
                    warn!("The data worker will outlive the page: {:?}", err);
                }

                host.link = Some(link);
                host.listeners.push(on_message);
                host.listeners.push(on_error);
                host.listeners.push(on_leave);
            }
            Err(err) => error!("The data worker could not be started: {:?}", err),
        }

        host
    }

    fn call(&self, request: Request, report: Option<Report>) -> Pending<Result<Reply, Fault>> {
        let Some(link) = &self.link else {
            return Box::pin(async { Err(Fault("the data worker is not running".to_owned())) });
        };
        let id = link.next.get();
        let (sender, receiver) = oneshot::channel();

        link.next.set(id.wrapping_add(1).max(HELLO + 1));
        self.calls.borrow_mut().insert(id, Waiting { reply: sender, report });

        let posted = protocol::encode(&Call { id, request })
            .ok_or_else(|| Fault("the request could not be encoded".to_owned()))
            .and_then(|bytes| link.post(Uint8Array::from(&bytes[..]).buffer()));

        if let Err(fault) = posted {
            self.calls.borrow_mut().remove(&id);

            return Box::pin(async move { Err(fault) });
        }

        Box::pin(async move { receiver.await.map_err(|_| Fault("the data worker went away".to_owned())) })
    }
}

fn failed(calls: &Calls, link: &Link, event: JsValue) {
    let reason = event.dyn_ref::<ErrorEvent>().map_or_else(
        || "the data worker failed to start".to_owned(),
        |event| format!("the data worker failed: {} ({}:{})", event.message(), event.filename(), event.lineno()),
    );

    error!("{reason}");
    *link.broken.borrow_mut() = Some(reason.clone());
    link.outbox.borrow_mut().clear();

    for (_, waiting) in calls.borrow_mut().drain() {
        let _ = waiting.reply.send(Reply::Failed(reason.clone()));
    }
}

fn received(calls: &Calls, link: &Link, event: MessageEvent) {
    let bytes = Uint8Array::new(&event.data()).to_vec();
    let Some(answer) = protocol::decode::<Answer>(&bytes) else {
        warn!("A message from the data worker could not be decoded");
        return;
    };

    if answer.id == HELLO {
        match answer.reply {
            Reply::Ready => link.flush(),
            Reply::Phase { label, done, total } => {
                let phase = Phase { label, done, total };

                if let Some(watch) = link.watch.borrow().as_ref() {
                    watch(phase.clone());
                }

                *link.phase.borrow_mut() = Some(phase);
            }
            Reply::Battle(event) => {
                if let BattleEvent::Standing(standing) = &event {
                    link.standing.set(standing.phase);
                }

                match event {
                    BattleEvent::Rolled(rolled) => film::deliver(rolled),
                    BattleEvent::Frame(_) | BattleEvent::Sheets(_) | BattleEvent::Audio(_) => link.screen.borrow_mut().handle(event),
                    other => {
                        link.screen.borrow_mut().handle(other.clone());

                        if let Some(watch) = link.battle_watch.borrow().as_ref() {
                            watch(other);
                        }
                    }
                }
            }
            _ => {}
        }

        return;
    }

    if let Reply::Progress { done, total } = answer.reply {
        if let Some(report) = calls.borrow().get(&answer.id).and_then(|waiting| waiting.report.as_ref()) {
            report(Progress { done, total });
        }

        return;
    }

    if let Some(waiting) = calls.borrow_mut().remove(&answer.id) {
        let _ = waiting.reply.send(answer.reply);
    }
}

pub(crate) fn logging() -> bool {
    storage().and_then(|storage| storage.get_item(LOGGING_KEY).ok().flatten()).as_deref() == Some(LOGGING_ON)
}

fn start() -> Result<Worker, JsValue> {
    let document = web_sys::window().and_then(|window| window.document()).ok_or_else(|| JsValue::from_str("no document"))?;
    let script: HtmlScriptElement = document
        .query_selector("script[type=module]")?
        .ok_or_else(|| JsValue::from_str("the page has no module script"))?
        .dyn_into()?;
    let glue = script.src();
    let worker_options = WorkerOptions::new();

    worker_options.set_type(WorkerType::Module);
    worker_options.set_name(if logging() { worker::LOGGED_NAME } else { worker::NAME });

    Worker::new_with_options(&glue, &worker_options)
}

const PAGE_HIDE: &str = "pagehide";
const FULLSCREEN_CHANGE: &str = "fullscreenchange";
const PERSISTED: &str = "persisted";
const SERVICE_WORKER: Asset = asset!("/assets/sw.js", AssetOptions::builder().with_hash_suffix(false));
const IMAGE_CACHE: &str = "bcc-images";

fn describe(err: JsValue) -> Fault {
    Fault(err.as_string().unwrap_or_else(|| format!("{err:?}")))
}

fn blob(bytes: &[u8], mime: &str) -> Result<Blob, JsValue> {
    let parts = Array::of1(&Uint8Array::from(bytes));
    let options = BlobPropertyBag::new();

    options.set_type(mime);
    Blob::new_with_u8_array_sequence_and_options(&parts, &options)
}

const SAVE_TYPE: &str = "application/octet-stream";
const REVOKE_DELAY_MS: i32 = 60_000;

fn save(name: &str, _mime: &str, bytes: &[u8]) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("no document"))?;
    let body = document.body().ok_or_else(|| JsValue::from_str("no body"))?;
    let blob = blob(bytes, SAVE_TYPE)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;

    anchor.set_href(&url);
    anchor.set_download(name);
    anchor.set_target("_self");
    anchor.set_rel("noopener");
    anchor.set_attribute("style", "display: none")?;
    body.append_child(&anchor)?;
    anchor.click();

    let cleanup = Closure::<dyn FnMut()>::once_into_js(move || {
        anchor.remove();
        let _ = Url::revoke_object_url(&url);
    });

    window.set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), REVOKE_DELAY_MS)?;

    Ok(())
}

async fn save_attachment(name: &str, bytes: &[u8]) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("no document"))?;
    let body = document.body().ok_or_else(|| JsValue::from_str("no body"))?;
    let caches = window.caches()?;
    let cache: Cache = JsFuture::from(caches.open(IMAGE_CACHE)).await?.dyn_into()?;
    let headers = Headers::new()?;

    headers.set("Content-Type", SAVE_TYPE)?;
    headers.set("Content-Disposition", &format!("attachment; filename=\"{}\"", name.replace('"', "")))?;
    headers.set("Cache-Control", "no-store")?;

    let init = ResponseInit::new();

    init.set_headers(&headers);

    let mut payload = bytes.to_vec();
    let response = Response::new_with_opt_u8_array_and_init(Some(&mut payload), &init)?;
    let path = format!("/images/downloads/{name}");

    JsFuture::from(cache.put_with_str(&path, &response)).await?;

    let frame = document.create_element("iframe")?;

    frame.set_attribute("style", "display: none")?;
    frame.set_attribute("src", &path)?;
    body.append_child(&frame)?;

    let cleanup = Closure::<dyn FnMut()>::once_into_js(move || frame.remove());

    window.set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), REVOKE_DELAY_MS)?;

    Ok(())
}

fn register_worker() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let container = window.navigator().service_worker();
    let options = RegistrationOptions::new();

    options.set_scope("/");

    let promise = container.register_with_options(&SERVICE_WORKER.to_string(), &options);

    wasm_bindgen_futures::spawn_local(async move {
        match JsFuture::from(promise).await {
            Ok(_) => info!("Image naming is available: the service worker controls the page"),
            Err(err) => info!("Images will carry generic names, the service worker could not be registered: {:?}", err),
        }
    });
}

fn controlled() -> bool {
    web_sys::window().is_some_and(|window| window.navigator().service_worker().controller().is_some())
}

async fn store_image(folder: &str, name: &str, png: &[u8]) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let caches = window.caches()?;
    let cache: Cache = JsFuture::from(caches.open(IMAGE_CACHE)).await?.dyn_into()?;
    let headers = Headers::new()?;

    headers.set("Content-Type", "image/png")?;
    headers.set("Cache-Control", "no-store")?;

    let init = ResponseInit::new();

    init.set_headers(&headers);

    let mut body = png.to_vec();
    let response = Response::new_with_opt_u8_array_and_init(Some(&mut body), &init)?;
    let path = format!("/images/{folder}/{name}");

    JsFuture::from(cache.put_with_str(&path, &response)).await?;

    Ok(path)
}

fn clipboard() -> Option<JsValue> {
    let navigator = web_sys::window()?.navigator();
    let clipboard = Reflect::get(&navigator, &JsValue::from_str("clipboard")).ok()?;

    (!clipboard.is_undefined() && !clipboard.is_null()).then_some(clipboard)
}

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn assemble(wire: RigWire) -> Option<RigParts> {
    let image = RgbaImage::from_raw(wire.width, wire.height, wire.rgba)?;
    let sheet = SpriteSheet {
        image_data: Some(Arc::new(image)),
        version: wire.version,
        image_name: wire.image_name,
        cuts: wire.cuts.into_iter().map(|(x, y, width, height, name)| SpriteCut { x, y, width, height, name }).collect(),
        opaque: wire.opaque.into_iter().map(|held| held.map(|(x, y, width, height)| Opaque { x, y, width, height })).collect(),
        hull: wire.hull,
    };

    Some(RigParts { sheet, model: Arc::from(wire.model) })
}

fn unexpected(reply: Reply) -> Fault {
    match reply {
        Reply::Failed(reason) => Fault(reason),
        _ => Fault("the data worker answered with something else".to_owned()),
    }
}

impl Host for WebHost {
    fn recall(&self, key: &str) -> Option<String> {
        storage()?.get_item(key).ok().flatten()
    }

    fn remember(&self, key: &str, value: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(key, value);
        }
    }

    fn pixel_ratio(&self) -> f32 {
        web_sys::window().map_or(1.0, |window| window.device_pixel_ratio() as f32).max(0.5)
    }

    fn frame(&self) -> Pending<f64> {
        let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
            let scheduled = web_sys::window().map(|window| window.request_animation_frame(&resolve));

            if !matches!(scheduled, Some(Ok(_))) {
                let _ = resolve.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
            }
        });

        Box::pin(async move { JsFuture::from(promise).await.ok().and_then(|value| value.as_f64()).unwrap_or(0.0) })
    }

    fn now(&self) -> f64 {
        web_sys::window().and_then(|window| window.performance()).map_or(0.0, |performance| performance.now())
    }

    fn font(&self, script: Script) -> Pending<Result<Bytes, Fault>> {
        let url = match script {
            Script::Japanese => &omote::theme::NOTO_JP,
            Script::Korean => &omote::theme::NOTO_KR,
            Script::TraditionalChinese => &omote::theme::NOTO_TC,
            Script::Thai => &omote::theme::NOTO_TH,
        }
        .to_string();

        Box::pin(async move {
            let window = web_sys::window().ok_or_else(|| Fault("no window".to_owned()))?;
            let response: Response = JsFuture::from(window.fetch_with_str(&url)).await.map_err(describe)?.dyn_into().map_err(describe)?;

            if !response.ok() {
                return Err(Fault(format!("[{}] {url}", response.status())));
            }

            let buffer = JsFuture::from(response.array_buffer().map_err(describe)?).await.map_err(describe)?;

            Ok(Arc::from(Uint8Array::new(&buffer).to_vec()))
        })
    }

    fn download(&self, name: &str, mime: &str, bytes: Bytes) {
        deliver_file(name, mime, &bytes);
    }

    fn can_copy(&self) -> bool {
        clipboard().is_some() && web_sys::window().is_some_and(|window| Reflect::has(&window, &JsValue::from_str("ClipboardItem")).unwrap_or(false))
    }

    fn copy_image(&self, png: Bytes) -> Pending<Result<(), Fault>> {
        Box::pin(async move {
            let clipboard = clipboard().ok_or_else(|| Fault("the clipboard is not available".to_owned()))?;
            let blob = blob(&png, "image/png").map_err(describe)?;
            let record = Object::new();

            Reflect::set(&record, &JsValue::from_str("image/png"), &blob).map_err(describe)?;

            let window = web_sys::window().ok_or_else(|| Fault("no window".to_owned()))?;
            let constructor: Function = Reflect::get(&window, &JsValue::from_str("ClipboardItem")).map_err(describe)?.dyn_into().map_err(describe)?;
            let item = Reflect::construct(&constructor, &Array::of1(&record)).map_err(describe)?;
            let write: Function = Reflect::get(&clipboard, &JsValue::from_str("write")).map_err(describe)?.dyn_into().map_err(describe)?;
            let promise: Promise = write.call1(&clipboard, &Array::of1(&item)).map_err(describe)?.dyn_into().map_err(describe)?;

            JsFuture::from(promise).await.map_err(describe)?;

            Ok(())
        })
    }

    fn copy_text(&self, text: &str) -> Pending<Result<(), Fault>> {
        let text = text.to_owned();

        Box::pin(async move {
            let clipboard = clipboard().ok_or_else(|| Fault("the clipboard is not available".to_owned()))?;
            let write: Function = Reflect::get(&clipboard, &JsValue::from_str("writeText")).map_err(describe)?.dyn_into().map_err(describe)?;
            let promise: Promise = write.call1(&clipboard, &JsValue::from_str(&text)).map_err(describe)?.dyn_into().map_err(describe)?;

            JsFuture::from(promise).await.map_err(describe)?;

            Ok(())
        })
    }

    fn pick_file(&self, accept: &str, limit: u64) -> Pending<Result<Option<Picked>, Fault>> {
        let opened = open_picker(accept);

        Box::pin(async move {
            let (input, chosen) = opened?;
            let settled = JsFuture::from(chosen).await;

            input.remove();
            settled.map_err(describe)?;

            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return Ok(None);
            };
            let size = file.size() as u64;

            if size > limit {
                return Ok(Some(Picked { name: file.name(), size, bytes: None }));
            }

            let buffer = JsFuture::from(file.array_buffer()).await.map_err(describe)?;

            Ok(Some(Picked { name: file.name(), size, bytes: Some(Arc::from(Uint8Array::new(&buffer).to_vec())) }))
        })
    }

    fn capture_pointer(&self, element: &str, pointer: i32) {
        let found = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id(element));

        if let Some(found) = found
            && let Err(err) = found.set_pointer_capture(pointer)
        {
            warn!("The pointer could not be captured by {element}: {:?}", err);
        }
    }

    fn can_publish(&self) -> bool {
        controlled()
    }

    fn publish_image(&self, folder: &str, name: &str, png: Bytes) -> Pending<Result<Option<String>, Fault>> {
        let (folder, name) = (folder.to_owned(), name.to_owned());

        Box::pin(async move {
            if !controlled() {
                return Ok(None);
            }

            store_image(&folder, &name, &png).await.map(Some).map_err(describe)
        })
    }

    fn sleep(&self, millis: u32) -> Pending<()> {
        let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
            let scheduled = web_sys::window().map(|window| window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis as i32));

            if !matches!(scheduled, Some(Ok(_))) {
                let _ = resolve.call0(&JsValue::NULL);
            }
        });

        Box::pin(async move {
            let _ = JsFuture::from(promise).await;
        })
    }

    fn surface(&self, canvas: &str, overlay: &str) -> Pending<Result<Rc<dyn Surface>, Fault>> {
        let (canvas, overlay) = (canvas.to_owned(), overlay.to_owned());

        Box::pin(async move { viewer::open(&canvas, &overlay).await.map(|surface| Rc::new(surface) as Rc<dyn Surface>) })
    }

    fn names(&self, watch: Watch) -> Pending<Result<Vec<Box<str>>, Fault>> {
        if let Some(link) = &self.link {
            if let Some(phase) = link.phase.borrow().as_ref() {
                watch(phase.clone());
            }

            *link.watch.borrow_mut() = Some(watch);
        }

        let pending = self.call(Request::Names, None);

        Box::pin(async move {
            match pending.await? {
                Reply::Names(names) => Ok(names),
                other => Err(unexpected(other)),
            }
        })
    }

    fn fetch(&self, names: Vec<Box<str>>) -> Pending<Result<Fetched, Fault>> {
        let pending = self.call(Request::Fetch(names), None);

        Box::pin(async move {
            match pending.await? {
                Reply::Files(files) => Ok(files.into_iter().map(|(name, bytes)| (name, Arc::from(bytes))).collect()),
                other => Err(unexpected(other)),
            }
        })
    }

    fn picture(&self, kind: Picture, name: Box<str>) -> Pending<Result<Option<Prepared>, Fault>> {
        let kind = match kind {
            Picture::CatBanner => PictureKind::CatBanner,
            Picture::EnemyBanner => PictureKind::EnemyBanner,
            Picture::Trimmed => PictureKind::Trimmed,
            Picture::Boxed(canvas) => PictureKind::Boxed(canvas),
            Picture::Cut { sheet, icon } => PictureKind::Cut { sheet, icon },
        };
        let pending = self.call(Request::Picture { kind, name }, None);

        Box::pin(async move {
            match pending.await? {
                Reply::Picture(prepared) => Ok(prepared.map(|held| Prepared { width: held.width, height: held.height, png: Arc::from(held.png) })),
                other => Err(unexpected(other)),
            }
        })
    }

    fn warm(&self, names: Vec<Box<str>>) -> Pending<()> {
        let pending = self.call(Request::Warm(names), None);

        Box::pin(async move {
            let _ = pending.await;
        })
    }

    fn rig(&self, png: Box<str>, cut: Box<str>, model: Box<str>) -> Pending<Result<Option<RigParts>, Fault>> {
        let pending = self.call(Request::Rig { png, cut, model }, None);

        Box::pin(async move {
            match pending.await? {
                Reply::Rig(wire) => Ok(wire.and_then(assemble)),
                other => Err(unexpected(other)),
            }
        })
    }

    fn cats(&self, config: ScannerConfig, report: Report) -> Pending<Result<Vec<CatEntry>, Fault>> {
        let pending = self.call(Request::Cats(config), Some(report));

        Box::pin(async move {
            match pending.await? {
                Reply::Cats(cats) => Ok(cats),
                other => Err(unexpected(other)),
            }
        })
    }

    fn mods(&self, command: ModCommand, report: Report) -> Pending<Result<ModState, Fault>> {
        let pending = self.call(Request::Mods(command), Some(report));

        Box::pin(async move {
            match pending.await? {
                Reply::Mods(state) => Ok(state),
                other => Err(unexpected(other)),
            }
        })
    }

    fn enemies(&self, config: ScannerConfig, report: Report) -> Pending<Result<Vec<EnemyEntry>, Fault>> {
        let pending = self.call(Request::Enemies(config), Some(report));

        Box::pin(async move {
            match pending.await? {
                Reply::Enemies(enemies) => Ok(enemies),
                other => Err(unexpected(other)),
            }
        })
    }

    fn stages(&self, config: ScannerConfig, report: Report) -> Pending<Result<StageBundle, Fault>> {
        let pending = self.call(Request::Stages(config), Some(report));

        Box::pin(async move {
            match pending.await? {
                Reply::Stages(bundle) => Ok(*bundle),
                other => Err(unexpected(other)),
            }
        })
    }

    fn battle(&self, command: BattleCommand) {
        let Some(link) = &self.link else {
            return;
        };

        match command {
            BattleCommand::ExportVideo(filming) => film::start(*filming, wiring(link)),
            BattleCommand::AbortVideo => film::abort(),
            command => post_battle(link, command),
        }
    }

    fn can_film(&self) -> bool {
        film::supported()
    }

    fn fullscreen(&self) -> bool {
        fullscreen_now()
    }

    fn set_fullscreen(&self, on: bool) -> Result<(), Fault> {
        let document = document().ok_or_else(|| Fault("no document".to_owned()))?;

        if !on {
            document.exit_fullscreen();

            return Ok(());
        }

        document.document_element().ok_or_else(|| Fault("the page has no root element".to_owned()))?.request_fullscreen().map_err(describe)
    }

    fn fullscreen_watch(&self, watch: Box<dyn Fn(bool)>) {
        *self.fullscreen.borrow_mut() = Some(watch);
    }

    fn battle_watch(&self, watch: BattleWatch) {
        if let Some(link) = &self.link {
            *link.battle_watch.borrow_mut() = Some(watch);
        }
    }

    fn battle_canvas(&self, canvas: &str) {
        let Some(link) = &self.link else {
            return;
        };

        if canvas.is_empty() {
            link.screen.borrow_mut().detach();

            return;
        }

        Screen::attach(&link.screen, canvas);

        let presenter = Rc::clone(link);

        link.screen.borrow_mut().on_painted(Box::new(move || {
            if matches!(presenter.standing.get(), BattlePhase::Covering | BattlePhase::Loading) {
                post_battle(&presenter, BattleCommand::Presented);
            }
        }));

        if !link.painting.replace(true) {
            paint_loop(Rc::clone(link));
        }
    }

    fn battle_display(&self, scale: f32, padded: bool) {
        if let Some(link) = &self.link {
            link.screen.borrow_mut().display(scale, padded);
        }
    }
}

fn open_picker(accept: &str) -> Result<(HtmlInputElement, Promise), Fault> {
    let document = web_sys::window().and_then(|window| window.document()).ok_or_else(|| Fault("no document".to_owned()))?;
    let body = document.body().ok_or_else(|| Fault("the page has no body".to_owned()))?;
    let input: HtmlInputElement = document.create_element("input").map_err(describe)?.dyn_into().map_err(|element: web_sys::Element| describe(element.into()))?;

    input.set_type("file");
    input.set_accept(accept);
    input.set_attribute("style", "display: none").map_err(describe)?;
    body.append_child(&input).map_err(describe)?;

    let chosen = Promise::new(&mut |resolve: Function, _reject: Function| {
        input.set_onchange(Some(&resolve));

        if let Err(err) = input.add_event_listener_with_callback("cancel", &resolve) {
            warn!("The file picker cannot report a cancel: {:?}", err);
        }
    });

    input.click();

    Ok((input, chosen))
}

fn wiring(link: &Rc<Link>) -> film::Wiring {
    let (sender, teller) = (Rc::clone(link), Rc::clone(link));

    film::Wiring {
        send: Box::new(move |command| post_battle(&sender, command)),
        tell: Box::new(move |event| {
            if let Some(watch) = teller.battle_watch.borrow().as_ref() {
                watch(event);
            }
        }),
        save: Box::new(|bytes| deliver_file(film::FILE_NAME, film::MIME, &bytes)),
    }
}

fn deliver_file(name: &str, mime: &str, bytes: &[u8]) {
    let name = name.to_owned();

    if controlled() {
        let bytes = bytes.to_vec();

        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = save_attachment(&name, &bytes).await {
                error!("The download of {name} could not be started: {:?}", err);
            }
        });

        return;
    }

    if let Err(err) = save(&name, mime, bytes) {
        error!("The download of {name} could not be started: {:?}", err);
    }
}

fn post_battle(link: &Rc<Link>, command: BattleCommand) {
    let id = link.next.get();

    link.next.set(id.wrapping_add(1).max(HELLO + 1));

    let posted = protocol::encode(&Call { id, request: Request::Battle(command) })
        .ok_or_else(|| Fault("the battle command could not be encoded".to_owned()))
        .and_then(|bytes| link.post(Uint8Array::from(&bytes[..]).buffer()));

    if let Err(fault) = posted {
        warn!("A battle command could not be posted: {fault}");
    }
}

fn paint_loop(link: Rc<Link>) {
    let frame = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let hook = Rc::clone(&frame);
    let closure = Closure::<dyn FnMut(f64)>::new(move |_: f64| {
        battle::settle_audio(&link.screen);
        link.screen.borrow_mut().draw();

        let again = link.screen.borrow().attached();

        if again {
            if let (Some(window), Some(closure)) = (web_sys::window(), hook.borrow().as_ref()) {
                let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
            }
        } else {
            link.painting.set(false);
            hook.borrow_mut().take();
        }
    });

    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
    }

    *frame.borrow_mut() = Some(closure);
}
