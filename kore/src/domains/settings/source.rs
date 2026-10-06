mod ledger;
mod payload;

use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_TYPE, LAST_MODIFIED};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::common::architecture::{self, Workspace};
use crate::common::job::Ticker;

use ledger::{Held, Ledger};
use payload::{Chunk, Fault, Manifest};

const AGENT: &str = concat!("BattleCatsComplete/", env!("CARGO_PKG_VERSION"));
const HOSTS: [&str; 2] = ["drive.google.com", "docs.google.com"];
const DOWNLOAD_ROOT: &str = "https://drive.usercontent.google.com/download";
const LISTING_ROOT: &str = "https://drive.google.com/embeddedfolderview";
const MANIFEST: &str = "manifest.json";
const RETIRED: &str = "retired";
const STRANDED: &str = "game.previous";
const SHIFT_TRIES: u32 = 5;
const SHIFT_PAUSE: Duration = Duration::from_millis(200);
const ENTRY_MARK: &str = "id=\"entry-";
const TITLE_MARK: &str = "flip-entry-title\">";
const SHORTEST_ID: usize = 10;
const TIMEOUT: Duration = Duration::from_secs(20);
const CONNECTIONS: usize = 16;
const PULL_TRIES: u32 = 4;
const PULL_PAUSE: Duration = Duration::from_secs(1);

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
    Replacing,
}

struct Tally<'a, E> {
    done: AtomicU64,
    total: u64,
    ticker: Ticker,
    failed: AtomicBool,
    abort: &'a AtomicBool,
    emit: &'a E,
}

struct Feed<'a, E> {
    response: Response,
    tally: &'a Tally<'a, E>,
    taken: u64,
    cut: bool,
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
}

impl<E: Fn(Step) + Sync> Read for Feed<'_, E> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.tally.halted() {
            return Err(io::Error::other("the download was stopped"));
        }

        let read = self.response.read(buffer).inspect_err(|_| self.cut = true)?;

        self.taken += read as u64;
        self.tally.add(read as u64);

        Ok(read)
    }
}

fn parse(link: &str) -> Option<String> {
    let link = link.trim();
    let address = link.strip_prefix("https://").or_else(|| link.strip_prefix("http://")).unwrap_or(link);
    let (host, _) = address.split_once('/')?;

    if !HOSTS.contains(&host) {
        return None;
    }

    if let Some(id) = id_after(address, "/folders/") {
        return Some(id);
    }

    address.contains("folderview").then(|| id_after(address, "?id=").or_else(|| id_after(address, "&id="))).flatten()
}

pub fn current(remote: &Stamp, held: Option<&Stamp>) -> bool {
    same(remote, held) && architecture::game_present()
}

fn same(remote: &Stamp, held: Option<&Stamp>) -> bool {
    held.is_some_and(|held| held.id == remote.id)
}

pub fn probe(link: &str) -> Result<Stamp, Error> {
    inspect(link).map(|(stamp, _, _)| stamp).inspect_err(|err| warn!("Game data source could not be checked: {}", err))
}

pub fn install(link: &str, emit: impl Fn(Step) + Sync, abort: &AtomicBool) -> Result<Stamp, Error> {
    fetch(link, &emit, abort).inspect_err(|err| match err.kind {
        ErrorKind::Aborted => warn!("Game data download was cancelled"),
        _ => error!("Game data could not be installed from its source: {}", err),
    })
}

fn inspect(link: &str) -> Result<(Stamp, Manifest, Ledger), Error> {
    let folder = parse(link).ok_or_else(|| Error::new(ErrorKind::Link, "this is not a provider folder link".to_owned()))?;
    let client = client(Some(TIMEOUT), false)?;
    let file = locate(&client, &folder)?;
    let response = client.get(download_url(&file)).send().map_err(unreachable)?;

    vet(&response)?;

    let modified = response.headers().get(LAST_MODIFIED).and_then(|value| value.to_str().ok()).unwrap_or_default().to_owned();
    let bytes = response.bytes().map_err(unreachable)?;
    let manifest = Manifest::read(&bytes).map_err(malformed)?;
    let ledger = Ledger::load(Path::new(architecture::GAME));
    let stamp = Stamp { id: manifest.id.clone(), modified, size: ledger.missing(&manifest.chunks) };

    debug!(payload = stamp.id, modified = stamp.modified, size = stamp.size, "Game data source answered");

    Ok((stamp, manifest, ledger))
}

fn fetch(link: &str, emit: &(impl Fn(Step) + Sync), abort: &AtomicBool) -> Result<Stamp, Error> {
    let (stamp, manifest, ledger) = inspect(link)?;
    let game = Path::new(architecture::GAME);
    let (mut held, wanted) = ledger.split(game, &manifest.chunks);
    let workspace = Workspace::claim("source").map_err(disk)?;
    let staged = workspace.path().join(architecture::GAME);

    fs::create_dir_all(&staged).map_err(disk)?;
    info!(payload = stamp.id, chunks = manifest.chunks.len(), kept = held.len(), "Downloading game data from its source");

    let keep: HashSet<OsString> = held.iter().flat_map(|held| held.files.iter().map(|(name, _)| OsString::from(name))).collect();

    held.extend(download(&wanted, &staged, emit, abort)?);

    emit(Step::Replacing);
    replace(game, &staged, &workspace.path().join(RETIRED), &keep)?;
    Ledger::save(game, held);
    info!("Game data replaced from its source");

    Ok(stamp)
}

fn download(chunks: &[&Chunk], staged: &Path, emit: &(impl Fn(Step) + Sync), abort: &AtomicBool) -> Result<Vec<Held>, Error> {
    let client = client(None, true)?;
    let total = chunks.iter().map(|chunk| chunk.size).sum();
    let tally = Tally { done: AtomicU64::new(0), total, ticker: Ticker::default(), failed: AtomicBool::new(false), abort, emit };
    let next = AtomicUsize::new(0);
    let landed: Mutex<Vec<Held>> = Mutex::new(Vec::new());
    let failure: Mutex<Option<Error>> = Mutex::new(None);

    let work = || {
        while let Some(chunk) = chunks.get(next.fetch_add(1, Ordering::Relaxed)).filter(|_| !tally.halted()) {
            match pull(&client, chunk, staged, &tally) {
                Ok(files) => landed.lock().unwrap_or_else(PoisonError::into_inner).push(Held { name: chunk.name.clone(), files }),
                Err(err) => {
                    if !tally.halted() {
                        warn!("{} could not be downloaded: {}", chunk.name, err);
                    }

                    tally.failed.store(true, Ordering::Relaxed);
                    failure.lock().unwrap_or_else(PoisonError::into_inner).get_or_insert(err);

                    break;
                }
            }
        }
    };

    thread::scope(|scope| {
        for index in 1..CONNECTIONS.min(chunks.len()) {
            if let Err(err) = thread::Builder::new().name(format!("source_chunk_{index}")).spawn_scoped(scope, work) {
                warn!("A download connection could not be started, carrying on with fewer: {}", err);
            }
        }

        work();
    });

    if abort.load(Ordering::Relaxed) {
        return Err(aborted());
    }

    match failure.into_inner().unwrap_or_else(PoisonError::into_inner) {
        Some(err) => Err(err),
        None => Ok(landed.into_inner().unwrap_or_else(PoisonError::into_inner)),
    }
}

fn pull<E: Fn(Step) + Sync>(client: &Client, chunk: &Chunk, staged: &Path, tally: &Tally<E>) -> Result<Vec<(String, u64)>, Error> {
    let mut tries = 1;

    loop {
        match attempt(client, chunk, staged, tally) {
            Err(err) if err.kind == ErrorKind::Network && tries < PULL_TRIES && !tally.halted() => {
                warn!("{} was cut off, trying again: {}", chunk.name, err);
                thread::sleep(PULL_PAUSE * tries);
                tries += 1;
            }
            outcome => return outcome,
        }
    }
}

fn attempt<E: Fn(Step) + Sync>(client: &Client, chunk: &Chunk, staged: &Path, tally: &Tally<E>) -> Result<Vec<(String, u64)>, Error> {
    let response = client.get(download_url(&chunk.address)).send().map_err(unreachable)?;

    vet(&response)?;

    let mut feed = Feed { response, tally, taken: 0, cut: false };

    let err = match payload::unpack(&mut feed, staged) {
        Ok(files) if files.len() as u64 == chunk.files && feed.taken == chunk.size => return Ok(files),
        Ok(_) => malformed(format!("{} does not match the manifest", chunk.name)),
        Err(Fault::Disk(err)) => disk(err),
        Err(Fault::Stream(_)) if tally.abort.load(Ordering::Relaxed) => aborted(),
        Err(Fault::Stream(err)) if feed.cut || feed.taken < chunk.size => {
            Error::new(ErrorKind::Network, format!("{} stopped at {} of {} bytes: {}", chunk.name, feed.taken, chunk.size, err))
        }
        Err(Fault::Stream(err)) => malformed(format!("{} is not a readable chunk: {}", chunk.name, err)),
    };

    tally.rewind(feed.taken);

    Err(err)
}

fn replace(game: &Path, staged: &Path, retired: &Path, keep: &HashSet<OsString>) -> Result<(), Error> {
    fs::create_dir_all(game).map_err(disk)?;
    fs::create_dir_all(retired).map_err(disk)?;

    let held: Vec<OsString> = children(game)?.into_iter().filter(|name| !keep.contains(name)).collect();
    let fresh = children(staged)?;

    for (index, name) in held.iter().enumerate() {
        let Err(err) = shift(&game.join(name), &retired.join(name)) else {
            continue;
        };

        restore(&held[..index], retired, game);

        return Err(disk(err));
    }

    for (index, name) in fresh.iter().enumerate() {
        let Err(err) = shift(&staged.join(name), &game.join(name)) else {
            continue;
        };

        let stuck = fresh[..index].iter().filter(|name| !put_back(name, game, staged)).count();

        if stuck == 0 {
            restore(&held, retired, game);
        } else {
            strand(retired);
        }

        return Err(disk(err));
    }

    Ok(())
}

fn children(folder: &Path) -> Result<Vec<OsString>, Error> {
    fs::read_dir(folder).map_err(disk)?.map(|entry| entry.map(|entry| entry.file_name()).map_err(disk)).collect()
}

fn shift(from: &Path, to: &Path) -> io::Result<()> {
    let mut tries = 1;

    loop {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(err) if tries >= SHIFT_TRIES => return Err(err),
            Err(err) => {
                warn!("{} is held by something, trying again: {}", from.display(), err);
                thread::sleep(SHIFT_PAUSE);
                tries += 1;
            }
        }
    }
}

fn put_back(name: &OsString, from: &Path, to: &Path) -> bool {
    shift(&from.join(name), &to.join(name))
        .inspect_err(|err| error!("{} could not be moved back out of {}: {}", name.to_string_lossy(), from.display(), err))
        .is_ok()
}

fn restore(names: &[OsString], retired: &Path, game: &Path) {
    let lost = names.iter().filter(|name| !put_back(name, retired, game)).count();

    if lost > 0 {
        strand(retired);
    }
}

fn strand(retired: &Path) {
    match fs::rename(retired, STRANDED) {
        Ok(()) => error!("The previous game data could not all be put back, what is left of it was kept in {}", STRANDED),
        Err(err) => error!("The previous game data could not be put back or kept aside from {}: {}", retired.display(), err),
    }
}

fn locate(client: &Client, folder: &str) -> Result<String, Error> {
    let response = client.get(format!("{}?id={}", LISTING_ROOT, folder)).send().map_err(unreachable)?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err(Error::new(ErrorKind::Missing, "the shared folder no longer exists".to_owned()));
    }

    if !response.status().is_success() {
        return Err(Error::new(ErrorKind::Network, format!("the shared folder answered {}", response.status())));
    }

    let page = response
        .text()
        .map_err(|err| Error::new(ErrorKind::Network, format!("the shared folder could not be read: {}", err.without_url())))?;

    pick(&page).ok_or_else(|| Error::new(ErrorKind::Missing, "the shared folder holds no payload manifest".to_owned()))
}

fn pick(page: &str) -> Option<String> {
    page.split(ENTRY_MARK).skip(1).find_map(|entry| {
        let (id, rest) = entry.split_once('"')?;
        let (title, _) = rest.split_once(TITLE_MARK)?.1.split_once('<')?;

        title.eq_ignore_ascii_case(MANIFEST).then(|| id.to_owned())
    })
}

fn vet(response: &Response) -> Result<(), Error> {
    let status = response.status();

    if status == StatusCode::NOT_FOUND {
        return Err(Error::new(ErrorKind::Missing, "the shared payload no longer exists".to_owned()));
    }

    if !status.is_success() {
        return Err(Error::new(ErrorKind::Network, format!("the provider answered {}", status)));
    }

    let page = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|kind| kind.starts_with("text/html"));

    if page {
        return Err(Error::new(ErrorKind::Missing, "the folder is not shared with anyone who has the link".to_owned()));
    }

    Ok(())
}

fn client(timeout: Option<Duration>, separate: bool) -> Result<Client, Error> {
    let builder = Client::builder().user_agent(AGENT).connect_timeout(TIMEOUT).timeout(timeout);
    let builder = if separate { builder.http1_only() } else { builder };

    builder.build().map_err(|err| Error::new(ErrorKind::Network, format!("could not build the http client: {}", err)))
}

fn download_url(file: &str) -> String {
    format!("{}?id={}&export=download&confirm=t", DOWNLOAD_ROOT, file)
}

fn id_after(address: &str, marker: &str) -> Option<String> {
    let (_, rest) = address.split_once(marker)?;
    let id: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')).collect();

    (id.len() >= SHORTEST_ID).then_some(id)
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
    Error::new(ErrorKind::Aborted, "the download was cancelled".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FOLDER: &str = "1l_5RK28JRL19wpT22B-DY9We3TVXnnQQ";

    #[test]
    fn every_folder_link_shape_resolves_to_its_id() {
        let folder = Some(FOLDER.to_owned());

        assert_eq!(parse(&format!("https://drive.google.com/drive/folders/{FOLDER}?usp=drive_link")), folder);
        assert_eq!(parse(&format!("  https://drive.google.com/drive/u/0/folders/{FOLDER}  ")), folder);
        assert_eq!(parse(&format!("drive.google.com/embeddedfolderview?id={FOLDER}#list")), folder);
    }

    // A single shared file used to be a payload too, now only a folder is.
    #[test]
    fn anything_that_is_not_a_drive_folder_is_refused() {
        assert_eq!(parse(""), None);
        assert_eq!(parse(&format!("https://example.com/drive/folders/{FOLDER}")), None);
        assert_eq!(parse("https://drive.google.com/drive/folders/short"), None);
        assert_eq!(parse("https://drive.google.com/drive/my-drive"), None);
        assert_eq!(parse(&format!("https://drive.google.com/file/d/{FOLDER}/view?usp=sharing")), None);
        assert_eq!(parse(&format!("https://drive.google.com/uc?export=download&id={FOLDER}")), None);
    }

    #[test]
    fn a_folder_listing_gives_up_its_manifest_and_nothing_else() {
        let entry = |id: &str, title: &str| format!("<div class=\"flip-entry\" id=\"entry-{id}\"><div class=\"flip-entry-title\">{title}</div></div>");
        let page = [entry("aaa", "000.chunk"), entry("bbb", "Manifest.json"), entry("ccc", "001.chunk")].concat();

        assert_eq!(pick(&page).as_deref(), Some("bbb"));
        assert_eq!(pick(&[entry("aaa", "000.chunk"), entry("bbb", "game.zip")].concat()), None);
        assert_eq!(pick("<html>Sign in</html>"), None);
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
    fn a_swap_changes_what_the_folder_holds_and_keeps_the_folder() {
        let root = std::env::temp_dir().join(format!("bcc-source-swap-{}", std::process::id()));
        let (game, staged, retired) = (root.join("game"), root.join("staged"), root.join("retired"));
        let _ = fs::remove_dir_all(&root);

        fs::create_dir_all(game.join("cats")).unwrap();
        fs::write(game.join("cats/old.csv"), "old").unwrap();
        fs::write(game.join("gone.txt"), "gone").unwrap();
        fs::write(game.join("kept.png"), "kept").unwrap();
        fs::create_dir_all(&staged).unwrap();
        fs::write(staged.join("new.csv"), "new").unwrap();

        let before = (identity(&game), identity(&game.join("kept.png")));
        replace(&game, &staged, &retired, &HashSet::from([OsString::from("kept.png")])).unwrap();

        assert_eq!(fs::read_to_string(game.join("new.csv")).unwrap(), "new");
        assert!(!game.join("cats").exists() && !game.join("gone.txt").exists());
        assert!(retired.join("cats/old.csv").exists() && retired.join("gone.txt").exists());

        // A file from a chunk that did not change is never moved, only what is around it.
        assert_eq!((identity(&game), identity(&game.join("kept.png"))), before);
        assert!(!retired.join("kept.png").exists());

        let _ = fs::remove_dir_all(&root);
    }
}
