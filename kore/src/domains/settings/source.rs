use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, LAST_MODIFIED};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use zip::ZipArchive;

use crate::common::architecture::{self, Workspace};
use crate::common::job::Ticker;

const AGENT: &str = concat!("BattleCatsComplete/", env!("CARGO_PKG_VERSION"));
const HOSTS: [&str; 3] = ["drive.google.com", "docs.google.com", "drive.usercontent.google.com"];
const DOWNLOAD_ROOT: &str = "https://drive.usercontent.google.com/download";
const LISTING_ROOT: &str = "https://drive.google.com/embeddedfolderview";
const ARCHIVE: &str = "game.zip";
const RETIRED: &str = "retired";
const STRANDED: &str = "game.previous";
const SHIFT_TRIES: u32 = 5;
const SHIFT_PAUSE: Duration = Duration::from_millis(200);
const JUNK: &str = "__MACOSX";
const ENTRY_MARK: &str = "id=\"entry-";
const TITLE_MARK: &str = "flip-entry-title\">";
const SHORTEST_ID: usize = 10;
const CHUNK: usize = 256 * 1024;
const TIMEOUT: Duration = Duration::from_secs(20);

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    File(String),
    Folder(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Stamp {
    pub file: String,
    pub modified: String,
    pub size: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Step {
    Downloading { done: u64, total: u64 },
    Extracting { done: usize, total: usize },
    Replacing,
}

pub fn parse(link: &str) -> Option<Target> {
    let link = link.trim();
    let address = link.strip_prefix("https://").or_else(|| link.strip_prefix("http://")).unwrap_or(link);
    let (host, _) = address.split_once('/')?;

    if !HOSTS.contains(&host) {
        return None;
    }

    if let Some(id) = id_after(address, "/folders/") {
        return Some(Target::Folder(id));
    }

    if let Some(id) = id_after(address, "/file/d/") {
        return Some(Target::File(id));
    }

    let id = id_after(address, "?id=").or_else(|| id_after(address, "&id="))?;

    Some(if address.contains("folderview") { Target::Folder(id) } else { Target::File(id) })
}

pub fn current(remote: &Stamp, held: Option<&Stamp>) -> bool {
    held == Some(remote) && architecture::game_present()
}

pub fn probe(link: &str) -> Result<Stamp, Error> {
    inspect(link).inspect_err(|err| warn!("Game data source could not be checked: {}", err))
}

pub fn install(link: &str, emit: impl Fn(Step), abort: &AtomicBool) -> Result<Stamp, Error> {
    fetch(link, &emit, abort).inspect_err(|err| match err.kind {
        ErrorKind::Aborted => warn!("Game data download was cancelled"),
        _ => error!("Game data could not be installed from its source: {}", err),
    })
}

fn inspect(link: &str) -> Result<Stamp, Error> {
    let target = parse(link).ok_or_else(|| Error::new(ErrorKind::Link, "this is not a provider link".to_owned()))?;
    let client = client(Some(TIMEOUT))?;

    let file = match target {
        Target::File(id) => id,
        Target::Folder(id) => locate(&client, &id)?,
    };

    let response = client.head(download_url(&file)).send().map_err(unreachable)?;
    vet(&response)?;

    let headers = response.headers();
    let modified = headers.get(LAST_MODIFIED).and_then(|value| value.to_str().ok()).unwrap_or_default().to_owned();
    let size = headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);

    debug!(file, modified, size, "Game data source answered");

    Ok(Stamp { file, modified, size })
}

fn fetch(link: &str, emit: &impl Fn(Step), abort: &AtomicBool) -> Result<Stamp, Error> {
    let stamp = inspect(link)?;
    let workspace = Workspace::claim("source").map_err(disk)?;
    let archive = workspace.path().join(ARCHIVE);
    let staged = workspace.path().join(architecture::GAME);

    info!(file = stamp.file, size = stamp.size, "Downloading game data from its source");
    download(&stamp, &archive, emit, abort)?;
    extract(&archive, &staged, emit, abort)?;

    emit(Step::Replacing);
    replace(Path::new(architecture::GAME), &staged, &workspace.path().join(RETIRED))?;
    info!("Game data replaced from its source");

    Ok(stamp)
}

fn download(stamp: &Stamp, archive: &Path, emit: &impl Fn(Step), abort: &AtomicBool) -> Result<(), Error> {
    let client = client(None)?;
    let mut response = client.get(download_url(&stamp.file)).send().map_err(unreachable)?;
    vet(&response)?;

    let mut file = File::create(archive).map_err(disk)?;
    let mut buffer = vec![0u8; CHUNK];
    let mut done = 0u64;
    let pace = if stamp.size == 0 { usize::MAX } else { stamp.size as usize };
    let ticker = Ticker::default();

    loop {
        if abort.load(Ordering::Relaxed) {
            return Err(aborted());
        }

        let read = response
            .read(&mut buffer)
            .map_err(|err| Error::new(ErrorKind::Network, format!("the download was cut off: {}", err)))?;

        if read == 0 {
            break;
        }

        file.write_all(&buffer[..read]).map_err(disk)?;
        done += read as u64;

        if ticker.ready(done as usize, pace) {
            emit(Step::Downloading { done, total: stamp.size });
        }
    }

    if stamp.size != 0 && done != stamp.size {
        return Err(Error::new(ErrorKind::Network, format!("the download stopped at {} of {} bytes", done, stamp.size)));
    }

    Ok(())
}

fn extract(archive: &Path, staged: &Path, emit: &impl Fn(Step), abort: &AtomicBool) -> Result<(), Error> {
    let mut zip = ZipArchive::new(File::open(archive).map_err(disk)?).map_err(malformed)?;
    let total = zip.len();
    let wrapped = wrapped(zip.file_names());
    let ticker = Ticker::default();
    let mut written = 0usize;

    for index in 0..total {
        if abort.load(Ordering::Relaxed) {
            return Err(aborted());
        }

        let mut entry = zip.by_index(index).map_err(malformed)?;

        let Some(relative) = entry.enclosed_name().and_then(|name| placed(&name, wrapped)) else {
            continue;
        };

        let target = staged.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(disk)?;
            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(disk)?;
        }

        io::copy(&mut entry, &mut File::create(&target).map_err(disk)?).map_err(disk)?;
        written += 1;

        if ticker.ready(index + 1, total) {
            emit(Step::Extracting { done: index + 1, total });
        }
    }

    if written == 0 {
        return Err(Error::new(ErrorKind::Archive, "the archive holds no files".to_owned()));
    }

    Ok(())
}

fn replace(game: &Path, staged: &Path, retired: &Path) -> Result<(), Error> {
    fs::create_dir_all(game).map_err(disk)?;
    fs::create_dir_all(retired).map_err(disk)?;

    let held = children(game)?;
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

fn wrapped<'a>(mut names: impl Iterator<Item = &'a str>) -> bool {
    names.all(|name| top(name).is_some_and(|top| top.eq_ignore_ascii_case(architecture::GAME) || top == JUNK))
}

fn top(name: &str) -> Option<&str> {
    name.split(['/', '\\']).next().filter(|top| !top.is_empty())
}

fn placed(name: &Path, wrapped: bool) -> Option<PathBuf> {
    let mut parts = name.components();

    if name.starts_with(JUNK) {
        return None;
    }

    if wrapped {
        parts.next();
    }

    let relative = parts.as_path();

    (!relative.as_os_str().is_empty()).then(|| relative.to_path_buf())
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

    pick(&page).ok_or_else(|| Error::new(ErrorKind::Missing, "the shared folder holds no payload archive".to_owned()))
}

fn pick(page: &str) -> Option<String> {
    let archives: Vec<(&str, &str)> = page
        .split(ENTRY_MARK)
        .skip(1)
        .filter_map(|entry| {
            let (id, rest) = entry.split_once('"')?;
            let (title, _) = rest.split_once(TITLE_MARK)?.1.split_once('<')?;

            title.to_ascii_lowercase().ends_with(".zip").then_some((id, title))
        })
        .collect();

    archives
        .iter()
        .find(|(_, title)| title.eq_ignore_ascii_case(ARCHIVE))
        .or_else(|| archives.first())
        .map(|(id, _)| (*id).to_owned())
}

fn vet(response: &Response) -> Result<(), Error> {
    let status = response.status();

    if status == StatusCode::NOT_FOUND {
        return Err(Error::new(ErrorKind::Missing, "the shared file no longer exists".to_owned()));
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
        return Err(Error::new(ErrorKind::Missing, "the file is not shared with anyone who has the link".to_owned()));
    }

    Ok(())
}

fn client(timeout: Option<Duration>) -> Result<Client, Error> {
    Client::builder()
        .user_agent(AGENT)
        .connect_timeout(TIMEOUT)
        .timeout(timeout)
        .build()
        .map_err(|err| Error::new(ErrorKind::Network, format!("could not build the http client: {}", err)))
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

fn malformed(err: zip::result::ZipError) -> Error {
    Error::new(ErrorKind::Archive, format!("the download is not a readable zip: {}", err))
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

    const FILE: &str = "1l_5RK28JRL19wpT22B-DY9We3TVXnnQQ";

    #[test]
    fn every_share_link_shape_resolves_to_its_id() {
        let file = Some(Target::File(FILE.to_owned()));
        let folder = Some(Target::Folder(FILE.to_owned()));

        assert_eq!(parse(&format!("https://drive.google.com/file/d/{FILE}/view?usp=sharing")), file);
        assert_eq!(parse(&format!("  https://drive.google.com/open?id={FILE}  ")), file);
        assert_eq!(parse(&format!("drive.google.com/uc?export=download&id={FILE}")), file);
        assert_eq!(parse(&format!("https://drive.google.com/drive/folders/{FILE}?usp=drive_link")), folder);
        assert_eq!(parse(&format!("https://drive.google.com/drive/u/0/folders/{FILE}")), folder);
    }

    #[test]
    fn anything_that_is_not_a_drive_link_is_refused() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("https://example.com/file/d/1l_5RK28JRL19wpT22B/view"), None);
        assert_eq!(parse("https://drive.google.com/file/d/short/view"), None);
        assert_eq!(parse("https://drive.google.com/drive/my-drive"), None);
    }

    #[test]
    fn a_folder_listing_prefers_the_archive_named_for_the_game() {
        let entry = |id: &str, title: &str| format!("<div class=\"flip-entry\" id=\"entry-{id}\"><div class=\"flip-entry-title\">{title}</div></div>");
        let page = [entry("aaa", "notes.txt"), entry("bbb", "old.zip"), entry("ccc", "Game.zip")].concat();

        assert_eq!(pick(&page).as_deref(), Some("ccc"));
        assert_eq!(pick(&[entry("aaa", "notes.txt"), entry("bbb", "old.zip")].concat()).as_deref(), Some("bbb"));
        assert_eq!(pick(&entry("aaa", "notes.txt")), None);
    }

    // Zipping the folder itself and zipping its contents both have to land in the same place.
    #[test]
    fn a_wrapping_game_folder_is_peeled_off() {
        assert!(wrapped(["game/", "game/cats/000/unit001.csv", "__MACOSX/game/._cats"].into_iter()));
        assert!(!wrapped(["cats/000/unit001.csv", "game/tables/param.tsv"].into_iter()));

        assert_eq!(placed(Path::new("game/cats/unit001.csv"), true), Some(PathBuf::from("cats/unit001.csv")));
        assert_eq!(placed(Path::new("cats/unit001.csv"), false), Some(PathBuf::from("cats/unit001.csv")));
        assert_eq!(placed(Path::new("game"), true), None);
        assert_eq!(placed(Path::new("__MACOSX/game/._cats"), true), None);
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
        fs::create_dir_all(staged.join("cats")).unwrap();
        fs::write(staged.join("cats/new.csv"), "new").unwrap();

        let before = identity(&game);
        replace(&game, &staged, &retired).unwrap();

        assert_eq!(fs::read_to_string(game.join("cats/new.csv")).unwrap(), "new");
        assert!(!game.join("cats/old.csv").exists() && !game.join("gone.txt").exists());
        assert!(retired.join("cats/old.csv").exists() && retired.join("gone.txt").exists());
        assert_eq!(identity(&game), before);

        let _ = fs::remove_dir_all(&root);
    }
}
