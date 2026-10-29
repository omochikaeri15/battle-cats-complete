mod ledger;
mod payload;

use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_TYPE, LAST_MODIFIED, RANGE};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::common::architecture;
use crate::common::job::Ticker;

use ledger::{Held, Ledger};
use payload::{Chunk, Fault, Manifest};

const AGENT: &str = concat!("BattleCatsComplete/", env!("CARGO_PKG_VERSION"));
const SCHEME: &str = "https://";
const MANIFEST: &str = "manifest.json";
const TIMEOUT: Duration = Duration::from_secs(8);
const STALL_FLOOR: Duration = Duration::from_secs(30);
const SLOWEST_BYTES_PER_SEC: u64 = 32 * 1024;
const STALLS: u32 = 5;
const RESUME_PAUSE: Duration = Duration::from_secs(2);
const CONNECTIONS: usize = 16;
const SAMPLE_FILES: usize = 32;
const SLOW_FILE: Duration = Duration::from_millis(20);
const FAST_FILE: Duration = Duration::from_millis(4);
const PULL_TRIES: u32 = 3;
const PULL_PAUSE: Duration = Duration::from_secs(4);
const PAUSE_SLICE: Duration = Duration::from_millis(100);
const INSPECT_TRIES: u32 = 3;
const INSPECT_PAUSE: Duration = Duration::from_secs(1);
const WATCH_INTERVAL: Duration = Duration::from_secs(5);
const STALL_INTERVALS: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Link,
    Missing,
    Network,
    Archive,
    Disk,
    Aborted,
}

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    message: String,
}

impl Error {
    fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

pub struct Failure {
    pub error: Error,
    pub partial: Option<Partial>,
    pub touched: bool,
}

pub struct Partial {
    id: String,
    landed: Vec<Held>,
}

impl Partial {
    fn new(id: &str) -> Self {
        Self { id: id.to_owned(), landed: Vec::new() }
    }

    fn intact(self, game: &Path) -> Option<Self> {
        let whole = self.landed.iter().flat_map(|held| &held.files).all(|(name, size)| {
            fs::metadata(game.join(name)).is_ok_and(|meta| meta.is_file() && meta.len() == *size)
        });

        whole.then_some(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Stamp {
    pub id: String,
    pub modified: String,
    pub size: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Step {
    Downloading { done: u64, total: u64 },
    Unpacking { left: usize, done: u64, total: u64 },
    Replacing,
}

struct Gate {
    state: Mutex<(usize, usize)>,
    freed: Condvar,
}

struct Tally<'a, E> {
    done: AtomicU64,
    total: u64,
    unpacking: AtomicUsize,
    writers: Gate,
    ticker: Ticker,
    failed: AtomicBool,
    abort: &'a AtomicBool,
    emit: &'a E,
}

#[derive(Default)]
struct Lane {
    taken: AtomicU64,
    wrote: AtomicU64,
    busy: AtomicBool,
}

struct Feed<'a, E> {
    client: &'a Client,
    url: &'a str,
    name: &'a str,
    size: u64,
    response: Response,
    tally: &'a Tally<'a, E>,
    lane: &'a Lane,
    taken: u64,
    landmark: u64,
    stalls: u32,
}

impl Gate {
    fn new(limit: usize) -> Self {
        Self { state: Mutex::new((0, limit)), freed: Condvar::new() }
    }

    fn enter(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        while state.0 >= state.1 {
            state = self.freed.wait(state).unwrap_or_else(PoisonError::into_inner);
        }

        state.0 += 1;
    }

    fn leave(&self) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).0 -= 1;
        self.freed.notify_all();
    }

    fn tighten(&self) -> Option<usize> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        if state.1 <= 1 {
            return None;
        }

        state.1 = (state.1 / 2).max(1);

        Some(state.1)
    }

    fn loosen(&self, most: usize) -> Option<usize> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        if state.1 >= most {
            return None;
        }

        state.1 = (state.1 * 2).min(most);
        self.freed.notify_all();

        Some(state.1)
    }
}

impl<E: Fn(Step) + Sync> Tally<'_, E> {
    fn halted(&self) -> bool {
        self.failed.load(Ordering::Relaxed) || self.abort.load(Ordering::Relaxed)
    }

    fn add(&self, bytes: u64) {
        let done = self.done.fetch_add(bytes, Ordering::Relaxed) + bytes;

        if self.ticker.ready(done as usize, self.total as usize) {
            (self.emit)(Step::Downloading { done, total: self.total });
        }
    }

    fn rewind(&self, bytes: u64) {
        self.done.fetch_sub(bytes, Ordering::Relaxed);
    }

    fn pace(&self, files: usize, wrote: Duration) {
        if files < SAMPLE_FILES {
            return;
        }

        let per_file = wrote / files as u32;

        if per_file > SLOW_FILE && let Some(limit) = self.writers.tighten() {
            warn!("The disk needs {} ms per file, writing {} chunks at a time from now on", per_file.as_millis(), limit);
        } else if per_file < FAST_FILE && let Some(limit) = self.writers.loosen(CONNECTIONS) {
            info!("The disk is keeping up at {} ms per file, writing {} chunks at a time again", per_file.as_millis(), limit);
        }
    }

    fn unpacking(&self, entering: bool) {
        let left = if entering {
            self.unpacking.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            self.unpacking.fetch_sub(1, Ordering::Relaxed) - 1
        };

        let done = self.done.load(Ordering::Relaxed);

        if done >= self.total {
            (self.emit)(Step::Unpacking { left, done, total: self.total });
        }
    }

    fn pause(&self, wait: Duration) {
        let until = Instant::now() + wait;

        while !self.halted() && Instant::now() < until {
            thread::sleep(PAUSE_SLICE);
        }
    }
}

impl<E: Fn(Step) + Sync> Feed<'_, E> {
    fn resume(&mut self, err: io::Error) -> io::Result<()> {
        if self.taken == 0 {
            return Err(err);
        }

        if self.taken == self.landmark {
            self.stalls += 1;

            if self.stalls > STALLS {
                return Err(err);
            }

            warn!("{} stalled at {} of {} bytes, reconnecting ({} of {}): {}", self.name, self.taken, self.size, self.stalls, STALLS, err);
            self.tally.pause(RESUME_PAUSE);
        } else {
            self.stalls = 0;
            debug!("{} was cut off at {} of {} bytes while still moving, reconnecting: {}", self.name, self.taken, self.size, err);
        }

        self.landmark = self.taken;

        let response = self
            .client
            .get(self.url)
            .header(RANGE, format!("bytes={}-", self.taken))
            .timeout(deadline(self.size.saturating_sub(self.taken)))
            .send()
            .map_err(io::Error::other)?;

        debug!(chunk = self.name, status = %response.status(), length = response.content_length(), "A chunk was resumed");

        if response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(io::Error::other(explain(response.status())));
        }

        self.response = response;

        Ok(())
    }
}

impl<E: Fn(Step) + Sync> Read for Feed<'_, E> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.tally.halted() {
                return Err(io::Error::other("the download was stopped"));
            }

            let outcome = match self.response.read(buffer) {
                Ok(0) if self.taken < self.size => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "the stream ended early")),
                outcome => outcome,
            };

            match outcome {
                Ok(read) => {
                    self.taken += read as u64;
                    self.lane.taken.fetch_add(read as u64, Ordering::Relaxed);
                    self.tally.add(read as u64);

                    return Ok(read);
                }
                Err(err) => self.resume(err)?,
            }
        }
    }
}

fn parse(link: &str) -> Option<String> {
    let link = link.trim().trim_end_matches('/');
    let link = link.strip_suffix(MANIFEST).map_or(link, |base| base.trim_end_matches('/'));
    let rest = link.strip_prefix(SCHEME).unwrap_or(link);
    let host = rest.split('/').next().unwrap_or_default();
    let plain = host.contains('.') && !host.ends_with('.') && !host.contains(':') && !rest.contains(['?', '#', ' ', '\\']);

    plain.then(|| format!("{SCHEME}{rest}"))
}

pub fn current(remote: &Stamp, held: Option<&Stamp>) -> bool {
    same(remote, held) && architecture::game_present()
}

fn same(remote: &Stamp, held: Option<&Stamp>) -> bool {
    held.is_some_and(|held| held.id == remote.id)
}

pub fn probe(link: &str) -> Result<Stamp, Error> {
    inspect(link).map(|(stamp, ..)| stamp).inspect_err(|err| warn!("Game data source could not be checked: {}", err))
}

pub fn install(link: &str, partial: Option<Partial>, emit: impl Fn(Step) + Sync, abort: &AtomicBool) -> Result<Stamp, Failure> {
    fetch(link, partial, &emit, abort).inspect_err(|failure| match failure.error.kind {
        ErrorKind::Aborted => warn!("Game data download was canceled"),
        _ if failure.partial.is_some() => error!("Game data download stopped and can be resumed: {}", failure.error),
        _ => error!("Game data could not be installed from its source: {}", failure.error),
    })
}

fn inspect(link: &str) -> Result<(Stamp, Manifest, Ledger, String), Error> {
    let base = parse(link).ok_or_else(|| Error::new(ErrorKind::Link, "this is not an https link to a game data source".to_owned()))?;
    let client = client(Some(TIMEOUT), false)?;
    let (modified, bytes) = retry(|| {
        let url = url(&base, MANIFEST);
        let response = client.get(&url).send().map_err(unreachable)?;

        debug!(url, status = %response.status(), length = response.content_length(), "The manifest request was answered");

        vet(&response, Error::new(ErrorKind::Missing, "the link answers with a page instead of a payload".to_owned()))?;

        let modified = response.headers().get(LAST_MODIFIED).and_then(|value| value.to_str().ok()).unwrap_or_default().to_owned();
        let bytes = response.bytes().map_err(unreachable)?;

        Ok((modified, bytes))
    })?;
    let manifest = Manifest::read(&bytes).map_err(malformed)?;
    let ledger = Ledger::load();
    let stamp = Stamp { id: manifest.id.clone(), modified, size: ledger.missing(&manifest.chunks) };

    debug!(payload = stamp.id, modified = stamp.modified, size = stamp.size, "Game data source answered");

    Ok((stamp, manifest, ledger, base))
}

fn retry<T>(mut step: impl FnMut() -> Result<T, Error>) -> Result<T, Error> {
    let mut tries = 1;

    loop {
        match step() {
            Err(err) if err.kind == ErrorKind::Network && tries < INSPECT_TRIES => {
                warn!("The game data source did not answer, trying again: {}", err);
                thread::sleep(INSPECT_PAUSE * tries);
                tries += 1;
            }
            outcome => return outcome,
        }
    }
}

fn fetch(link: &str, partial: Option<Partial>, emit: &(impl Fn(Step) + Sync), abort: &AtomicBool) -> Result<Stamp, Failure> {
    let (stamp, manifest, ledger, base) = match inspect(link) {
        Ok(found) => found,
        Err(error) => return Err(Failure { error, partial, touched: false }),
    };
    let game = Path::new(architecture::GAME);
    let (mut held, wanted) = ledger.split(game, &manifest.chunks);
    let mut partial = match partial {
        Some(partial) if partial.id == stamp.id => partial.intact(game).unwrap_or_else(|| {
            info!("Files from the interrupted download were changed since, starting the download over");

            Partial::new(&stamp.id)
        }),
        Some(_) => {
            info!("The game data source was republished, starting the download over");

            Partial::new(&stamp.id)
        }
        None => Partial::new(&stamp.id),
    };
    let (wanted, resumed) = outstanding(wanted, &partial.landed);

    info!(payload = stamp.id, chunks = manifest.chunks.len(), kept = held.len(), landed = partial.landed.len(), "Downloading game data from its source");
    debug!(wanted = wanted.len(), bytes = wanted.iter().map(|chunk| chunk.size).sum::<u64>(), resumed, "Chunks still to fetch");

    if let Err(err) = fs::create_dir_all(game) {
        return Err(Failure { error: disk(err), partial: Some(partial), touched: false });
    }

    if let Err(error) = download(&base, &wanted, game, resumed, &mut partial.landed, emit, abort) {
        let partial = (error.kind != ErrorKind::Aborted).then_some(partial);

        return Err(Failure { error, partial, touched: true });
    }

    emit(Step::Replacing);

    held.extend(partial.landed);

    let current: HashSet<OsString> = held.iter().flat_map(|held| &held.files).filter_map(|(name, _)| top(name)).collect();

    Ledger::save(game, held);
    sweep(game, &current);
    info!("Game data installed from its source");

    Ok(stamp)
}

fn outstanding<'a>(wanted: Vec<&'a Chunk>, landed: &[Held]) -> (Vec<&'a Chunk>, u64) {
    let done: HashSet<&str> = landed.iter().map(|held| held.name.as_str()).collect();
    let (landed, wanted): (Vec<&Chunk>, Vec<&Chunk>) = wanted.into_iter().partition(|chunk| done.contains(chunk.name.as_str()));

    (wanted, landed.iter().map(|chunk| chunk.size).sum())
}

fn download(
    base: &str,
    chunks: &[&Chunk],
    game: &Path,
    resumed: u64,
    landed: &mut Vec<Held>,
    emit: &(impl Fn(Step) + Sync),
    abort: &AtomicBool,
) -> Result<(), Error> {
    let client = client(None, true)?;
    let total = resumed + chunks.iter().map(|chunk| chunk.size).sum::<u64>();
    let tally = Tally {
        done: AtomicU64::new(resumed),
        total,
        unpacking: AtomicUsize::new(0),
        writers: Gate::new(CONNECTIONS),
        ticker: Ticker::default(),
        failed: AtomicBool::new(false),
        abort,
        emit,
    };
    let next = AtomicUsize::new(0);
    let active = AtomicUsize::new(1);
    let lanes: Vec<Lane> = chunks.iter().map(|_| Lane::default()).collect();
    let fresh: Mutex<Vec<Held>> = Mutex::new(Vec::new());
    let failure: Mutex<Option<Error>> = Mutex::new(None);

    emit(Step::Downloading { done: resumed, total });

    let work = || {
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(chunk) = chunks.get(index).filter(|_| !tally.halted()) else {
                break;
            };

            match pull(&client, base, chunk, game, &tally, &lanes[index]) {
                Ok(files) => fresh.lock().unwrap_or_else(PoisonError::into_inner).push(Held { name: chunk.name.clone(), files }),
                Err(err) => {
                    if !tally.halted() {
                        warn!("{} could not be downloaded: {}", chunk.name, err);
                        failure.lock().unwrap_or_else(PoisonError::into_inner).get_or_insert(err);
                    }

                    tally.failed.store(true, Ordering::Relaxed);

                    break;
                }
            }
        }

        active.fetch_sub(1, Ordering::Relaxed);
    };

    thread::scope(|scope| {
        let watcher = thread::Builder::new().name("source_watch".to_owned()).spawn_scoped(scope, || watch(chunks, &lanes, &tally, &active));

        if let Err(err) = watcher {
            debug!("The download watchdog could not be started: {}", err);
        }

        for index in 1..CONNECTIONS.min(chunks.len()) {
            active.fetch_add(1, Ordering::Relaxed);

            if let Err(err) = thread::Builder::new().name(format!("source_chunk_{index}")).spawn_scoped(scope, work) {
                active.fetch_sub(1, Ordering::Relaxed);
                warn!("A download connection could not be started, carrying on with fewer: {}", err);
            }
        }

        debug!(connections = active.load(Ordering::Relaxed), chunks = chunks.len(), "Download connections opened");

        work();
    });

    landed.append(&mut fresh.into_inner().unwrap_or_else(PoisonError::into_inner));

    debug!(done = tally.done.load(Ordering::Relaxed), total, landed = landed.len(), failed = tally.failed.load(Ordering::Relaxed), "Download finished");

    if abort.load(Ordering::Relaxed) {
        return Err(aborted());
    }

    failure.into_inner().unwrap_or_else(PoisonError::into_inner).map_or(Ok(()), Err)
}

fn watch<E: Fn(Step) + Sync>(chunks: &[&Chunk], lanes: &[Lane], tally: &Tally<E>, active: &AtomicUsize) {
    let mut seen = vec![(0u64, 0u32); lanes.len()];
    let mut last = Instant::now();

    while active.load(Ordering::Relaxed) > 0 {
        thread::sleep(PAUSE_SLICE);

        if last.elapsed() < WATCH_INTERVAL {
            continue;
        }

        last = Instant::now();

        let mut flight = Vec::new();
        let mut receiving = 0;

        for (index, lane) in lanes.iter().enumerate() {
            if !lane.busy.load(Ordering::Relaxed) {
                seen[index] = (0, 0);
                continue;
            }

            let taken = lane.taken.load(Ordering::Relaxed);
            let chunk = chunks[index];

            if taken >= chunk.size {
                seen[index] = (0, 0);
                flight.push(format!("{} unpacking", chunk.name));
                continue;
            }

            let (before, quiet) = seen[index];
            let quiet = if taken == before { quiet + 1 } else { 0 };

            receiving += 1;
            seen[index] = (taken, quiet);
            flight.push(format!("{} {}/{}", chunk.name, taken, chunk.size));

            if quiet > 0 && quiet % STALL_INTERVALS == 0 {
                warn!(
                    "{} has not moved for {}s, stuck at {} of {} bytes",
                    chunk.name,
                    WATCH_INTERVAL.as_secs() * u64::from(quiet),
                    taken,
                    chunk.size
                );
            }
        }

        let (done, writing) = (tally.done.load(Ordering::Relaxed), tally.unpacking.load(Ordering::Relaxed));

        debug!(done, total = tally.total, connections = flight.len(), writing, "Download progress: {}", flight.join(", "));

        if receiving == 0 && writing > 0 {
            (tally.emit)(Step::Unpacking { left: writing, done, total: tally.total });
        }
    }
}

fn pull<E: Fn(Step) + Sync>(
    client: &Client,
    base: &str,
    chunk: &Chunk,
    game: &Path,
    tally: &Tally<E>,
    lane: &Lane,
) -> Result<Vec<(String, u64)>, Error> {
    let mut tries = 1;

    loop {
        match attempt(client, base, chunk, game, tally, lane, tries) {
            Err(err) if err.kind == ErrorKind::Network && tries < PULL_TRIES && !tally.halted() => {
                warn!("{} was cut off, trying again: {}", chunk.name, err);
                tally.pause(PULL_PAUSE);
                tries += 1;
            }
            outcome => return outcome,
        }
    }
}

fn attempt<E: Fn(Step) + Sync>(
    client: &Client,
    base: &str,
    chunk: &Chunk,
    game: &Path,
    tally: &Tally<E>,
    lane: &Lane,
    tries: u32,
) -> Result<Vec<(String, u64)>, Error> {
    let started = Instant::now();

    lane.taken.store(0, Ordering::Relaxed);
    lane.wrote.store(0, Ordering::Relaxed);
    lane.busy.store(true, Ordering::Relaxed);

    let outcome = transfer(client, base, chunk, game, tally, lane);
    let taken = lane.taken.load(Ordering::Relaxed);
    let elapsed = started.elapsed();
    let rate = taken as f64 / 1024.0 / elapsed.as_secs_f64().max(0.001);

    lane.busy.store(false, Ordering::Relaxed);

    match &outcome {
        Ok(files) => debug!(
            chunk = chunk.name,
            files = files.len(),
            taken,
            ms = elapsed.as_millis(),
            wrote_ms = lane.wrote.load(Ordering::Relaxed),
            kbps = rate as u64,
            attempt = tries,
            "A chunk landed"
        ),
        Err(err) => debug!(chunk = chunk.name, taken, ms = elapsed.as_millis(), attempt = tries, kind = ?err.kind, "A chunk did not land: {}", err),
    }

    outcome
}

fn transfer<E: Fn(Step) + Sync>(
    client: &Client,
    base: &str,
    chunk: &Chunk,
    game: &Path,
    tally: &Tally<E>,
    lane: &Lane,
) -> Result<Vec<(String, u64)>, Error> {
    let url = url(base, &chunk.address);
    let response = client.get(&url).timeout(deadline(chunk.size)).send().map_err(unreachable)?;
    if let Some(announced) = response.content_length().filter(|length| *length != chunk.size) {
        warn!("{} is announced as {} bytes but the manifest says {}", chunk.name, announced, chunk.size);
    }

    vet(&response, Error::new(ErrorKind::Network, format!("the provider answered with a page instead of {}", chunk.name)))?;

    let mut feed = Feed { client, url: &url, name: &chunk.name, size: chunk.size, response, tally, lane, taken: 0, landmark: 0, stalls: 0 };
    let mut body = Vec::with_capacity(chunk.size as usize);
    let outcome = match feed.read_to_end(&mut body) {
        Err(_) if tally.abort.load(Ordering::Relaxed) => Err(aborted()),
        Err(err) => Err(Error::new(ErrorKind::Network, format!("{} stopped at {} of {} bytes: {}", chunk.name, feed.taken, chunk.size, err))),
        Ok(_) if feed.taken != chunk.size => Err(malformed(format!("{} arrived as {} bytes instead of {}", chunk.name, feed.taken, chunk.size))),
        Ok(_) => {
            tally.unpacking(true);
            tally.writers.enter();

            let started = Instant::now();
            let unpacked = match payload::unpack(&body[..], game) {
                Ok(files) if files.len() as u64 == chunk.files => Ok(files),
                Ok(_) => Err(malformed(format!("{} does not match the manifest", chunk.name))),
                Err(Fault::Disk(err)) => Err(disk(err)),
                Err(Fault::Stream(err)) => Err(malformed(format!("{} is not a readable chunk: {}", chunk.name, err))),
            };
            let wrote = started.elapsed();

            tally.writers.leave();
            tally.unpacking(false);
            lane.wrote.store(wrote.as_millis() as u64, Ordering::Relaxed);

            if let Ok(files) = &unpacked {
                tally.pace(files.len(), wrote);
            }

            unpacked
        }
    };

    if outcome.is_err() {
        tally.rewind(feed.taken);
    }

    outcome
}

fn sweep(game: &Path, current: &HashSet<OsString>) {
    let started = Instant::now();
    let Ok(names) = children(game).inspect_err(|err| warn!("The game folder could not be listed to clear out old files: {}", err)) else {
        return;
    };
    let stale: Vec<OsString> = names.into_iter().filter(|name| !current.contains(name)).collect();
    let left = stale
        .iter()
        .filter(|name| discard(&game.join(name)).inspect_err(|err| warn!("{} could not be cleared out: {}", name.to_string_lossy(), err)).is_err())
        .count();

    debug!(stale = stale.len(), left, ms = started.elapsed().as_millis(), "Cleared out what the new payload no longer holds");
}

fn top(name: &str) -> Option<OsString> {
    Path::new(name).components().next().map(|part| part.as_os_str().to_owned())
}

fn children(folder: &Path) -> Result<Vec<OsString>, Error> {
    fs::read_dir(folder).map_err(disk)?.map(|entry| entry.map(|entry| entry.file_name()).map_err(disk)).collect()
}

fn discard(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn vet(response: &Response, page: Error) -> Result<(), Error> {
    let status = response.status();

    if status == StatusCode::NOT_FOUND {
        return Err(Error::new(ErrorKind::Missing, explain(status)));
    }

    if !status.is_success() {
        return Err(Error::new(ErrorKind::Network, explain(status)));
    }

    let html = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|kind| kind.starts_with("text/html"));

    if html {
        return Err(page);
    }

    Ok(())
}

fn client(timeout: Option<Duration>, separate: bool) -> Result<Client, Error> {
    let builder = Client::builder().user_agent(AGENT).connect_timeout(TIMEOUT).timeout(timeout);
    let builder = if separate { builder.http1_only() } else { builder };

    builder.build().map_err(|err| Error::new(ErrorKind::Network, format!("could not build the http client: {}", err)))
}

fn url(base: &str, file: &str) -> String {
    format!("{base}/{file}")
}

fn deadline(bytes: u64) -> Duration {
    STALL_FLOOR + Duration::from_secs(bytes / SLOWEST_BYTES_PER_SEC)
}

fn explain(status: StatusCode) -> String {
    let meaning = match status.as_u16() {
        200 => "the source ignored the resume request and sent the whole file",
        206 => "the source sent a partial file",
        400 => "the source could not make sense of the request",
        401 | 403 => "the source refused the request, the bucket is not public",
        404 => "nothing is published at this link",
        405 => "the source does not allow this kind of request",
        408 => "the source gave up waiting for the request",
        416 => "the resume asked for bytes past the end of the file",
        429 => "the source is rate limiting this download",
        500 => "the source hit an internal error",
        502 => "the edge could not get an answer from the storage",
        503 => "the source is temporarily unavailable",
        504 => "the edge timed out waiting for the storage",
        520 => "the edge got an unknown error from the storage",
        521 | 523 => "the storage is unreachable from the edge",
        522 | 524 => "the edge timed out connecting to the storage",
        525 | 526 => "the certificate between the edge and the storage failed",
        530 => "the edge refused the request, check the custom domain",
        _ => status.canonical_reason().unwrap_or("an answer this app does not understand"),
    };

    format!("[{}] {}", status.as_u16(), meaning)
}

fn unreachable(err: reqwest::Error) -> Error {
    Error::new(ErrorKind::Network, format!("request to the provider failed: {}", err.without_url()))
}

fn malformed(reason: String) -> Error {
    Error::new(ErrorKind::Archive, reason)
}

fn disk(err: io::Error) -> Error {
    Error::new(ErrorKind::Disk, format!("the game data could not be written: {}", err))
}

fn aborted() -> Error {
    Error::new(ErrorKind::Aborted, "the download was canceled".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The link is only ever joined with a file name, so it has to be a bare https origin, optionally with a folder.
    #[test]
    fn every_source_link_shape_resolves_to_its_base() {
        let base = Some("https://source.example.com".to_owned());

        assert_eq!(parse("https://source.example.com"), base);
        assert_eq!(parse("source.example.com"), base);
        assert_eq!(parse("  https://source.example.com/  "), base);
        assert_eq!(parse("https://source.example.com/manifest.json"), base);
        assert_eq!(parse("https://source.example.com/payload/"), Some("https://source.example.com/payload".to_owned()));
        assert_eq!(url(&parse("https://source.example.com").unwrap(), "abc.chunk"), "https://source.example.com/abc.chunk");
    }

    #[test]
    fn anything_that_is_not_an_https_origin_is_refused() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("http://source.example.com"), None);
        assert_eq!(parse("https://localhost"), None);
        assert_eq!(parse("localhost:8080"), None);
        assert_eq!(parse("https://source.example.com/?id=abc"), None);
        assert_eq!(parse("https://source.example.com/#list"), None);
    }

    #[test]
    fn a_status_is_explained_by_its_code_and_meaning() {
        assert_eq!(explain(StatusCode::NOT_FOUND), "[404] nothing is published at this link");
        assert_eq!(explain(StatusCode::from_u16(522).unwrap()), "[522] the edge timed out connecting to the storage");
        assert_eq!(explain(StatusCode::IM_A_TEAPOT), "[418] I'm a teapot");
        assert!(!explain(StatusCode::OK).ends_with('.'));
    }

    // A resumed download only asks for the chunks that never landed, and the bar starts where it was.
    #[test]
    fn a_resumed_download_skips_what_already_landed() {
        let chunk = |name: &str, size: u64| -> Chunk {
            serde_json::from_str(&format!(r#"{{"name":"{name}","address":"abc","size":{size},"files":1}}"#)).unwrap()
        };
        let chunks = [chunk("000.chunk", 10), chunk("001.chunk", 20), chunk("002.chunk", 30)];
        let landed = vec![Held { name: "001.chunk".to_owned(), files: Vec::new() }];

        let (wanted, resumed) = outstanding(chunks.iter().collect(), &landed);

        assert_eq!(wanted.iter().map(|chunk| chunk.name.as_str()).collect::<Vec<_>>(), ["000.chunk", "002.chunk"]);
        assert_eq!(resumed, 20);
        assert_eq!(outstanding(chunks.iter().collect(), &[]).0.len(), 3);
    }

    // Republishing the same payload on a later date is not an update, and a stamp saved before payloads had ids is stale.
    #[test]
    fn a_held_stamp_is_compared_by_payload_and_not_by_date() {
        let stamp = |id: &str, modified: &str| Stamp { id: id.to_owned(), modified: modified.to_owned(), size: 7 };

        assert!(same(&stamp("abc", "Mon"), Some(&stamp("abc", "Tue"))));
        assert!(!same(&stamp("abc", "Mon"), Some(&stamp("abd", "Mon"))));
        assert!(!same(&stamp("abc", "Mon"), Some(&Stamp::default())));
        assert!(!same(&stamp("abc", "Mon"), None));
    }

    #[cfg(unix)]
    fn identity(folder: &Path) -> u64 {
        std::os::unix::fs::MetadataExt::ino(&fs::metadata(folder).unwrap())
    }

    #[cfg(not(unix))]
    fn identity(folder: &Path) -> u64 {
        u64::from(folder.is_dir())
    }

    // The folder itself has to stay put: the file watcher holds it, and a fresh folder would go unwatched.
    #[test]
    fn a_sweep_clears_out_what_the_payload_dropped_and_keeps_the_folder() {
        let game = std::env::temp_dir().join(format!("bcc-source-sweep-{}", std::process::id()));
        let _ = fs::remove_dir_all(&game);

        fs::create_dir_all(game.join("cats")).unwrap();
        fs::write(game.join("cats/old.csv"), "old").unwrap();
        fs::write(game.join("gone.txt"), "gone").unwrap();
        fs::write(game.join("kept.png"), "kept").unwrap();
        fs::write(game.join("new.csv"), "new").unwrap();

        let before = (identity(&game), identity(&game.join("kept.png")));
        let current: HashSet<OsString> = ["kept.png", "new.csv"].into_iter().filter_map(top).collect();

        sweep(&game, &current);

        assert!(!game.join("cats").exists() && !game.join("gone.txt").exists());
        assert_eq!(fs::read_to_string(game.join("new.csv")).unwrap(), "new");
        assert_eq!((identity(&game), identity(&game.join("kept.png"))), before);

        let _ = fs::remove_dir_all(&game);
    }
}
