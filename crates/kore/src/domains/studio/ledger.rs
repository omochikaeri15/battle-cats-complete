use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, warn};

use crate::common::io::hidden_temp;

const HOME: &str = ".history";
const FILE: &str = "stack.json";
const VERSION: u32 = 2;
const DEPTH: usize = 500;
const BUDGET: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Kind {
    Anim,
    Model,
    Cuts,
    Rig,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Seat {
    name: String,
    hash: String,
    size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    kind: Kind,
    files: Vec<Seat>,
}

#[derive(Serialize, Deserialize)]
struct Book {
    version: u32,
    head: Vec<Seat>,
    entries: Vec<Record>,
}

impl Default for Book {
    fn default() -> Self {
        Self { version: VERSION, head: Vec::new(), entries: Vec::new() }
    }
}

pub type Snapshot = Vec<(PathBuf, Vec<u8>)>;

pub struct Ledger {
    folder: PathBuf,
    book: Book,
}

impl Ledger {
    pub fn open(folder: &Path, files: &[PathBuf]) -> Ledger {
        let mut ledger = Ledger { folder: folder.to_path_buf(), book: Book::default() };

        let read = fs::read(folder.join(HOME).join(FILE));

        let Ok(bytes) = read else {
            ledger.sweep();

            return ledger;
        };

        let parsed = serde_json::from_slice::<Book>(&bytes)
            .inspect_err(|err| warn!(folder = %folder.display(), "Studio could not read the saved history: {}", err))
            .ok()
            .filter(|book| book.version == VERSION);

        let current = seats(files);

        match parsed {
            Some(book) if current.as_ref() == Some(&book.head) => ledger.book = book,
            Some(_) => {
                debug!(folder = %folder.display(), "Studio dropped a saved history the files no longer match");

                ledger.save();
            }
            None => ledger.save(),
        }

        ledger.sweep();
        ledger
    }

    pub fn push(&mut self, kind: Kind, files: &[(PathBuf, Vec<u8>)]) {
        let home = self.folder.join(HOME);

        if let Err(err) = fs::create_dir_all(&home) {
            warn!(folder = %home.display(), "Studio could not make the history folder: {}", err);

            return;
        }

        let mut seated = Vec::with_capacity(files.len());

        for (path, body) in files {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            let hash = digest(body);
            let blob = home.join(&hash);

            if !blob.is_file()
                && let Err(err) = fs::write(&blob, body)
            {
                warn!(path = %blob.display(), "Studio could not save a history snapshot: {}", err);

                return;
            }

            seated.push(Seat { name: name.to_owned(), hash, size: body.len() as u64 });
        }

        self.book.entries.push(Record { kind, files: seated });

        let spilled = self.trim(DEPTH, BUDGET);

        self.save();

        if spilled {
            self.sweep();
        }
    }

    fn trim(&mut self, depth: usize, budget: u64) -> bool {
        let mut spilled = false;

        while self.book.entries.len() > 1 && (self.book.entries.len() > depth || self.weight() > budget) {
            self.book.entries.remove(0);
            spilled = true;
        }

        spilled
    }

    fn weight(&self) -> u64 {
        let mut seen = HashSet::new();

        self.book
            .entries
            .iter()
            .flat_map(|record| &record.files)
            .filter(|seat| seen.insert(seat.hash.as_str()))
            .map(|seat| seat.size)
            .sum()
    }

    pub fn pop(&mut self) -> Option<(Kind, Snapshot)> {
        let record = self.book.entries.pop()?;
        let home = self.folder.join(HOME);

        let files: io::Result<Snapshot> = record
            .files
            .iter()
            .map(|seat| fs::read(home.join(&seat.hash)).map(|body| (self.folder.join(&seat.name), body)))
            .collect();

        self.save();
        self.sweep();

        files
            .inspect_err(|err| warn!(folder = %self.folder.display(), "Studio could not read a history snapshot: {}", err))
            .ok()
            .map(|files| (record.kind, files))
    }

    pub fn seal(&mut self, files: &[PathBuf]) {
        let Some(head) = seats(files) else {
            return;
        };

        if head == self.book.head {
            return;
        }

        self.book.head = head;
        self.save();
    }

    pub fn clear(&mut self) {
        self.book.entries.clear();
        self.save();
        self.sweep();
    }

    fn save(&self) {
        if !self.folder.is_dir() {
            return;
        }

        let home = self.folder.join(HOME);

        if self.book.entries.is_empty() {
            if let Err(err) = fs::remove_dir_all(&home)
                && err.kind() != io::ErrorKind::NotFound
            {
                warn!(folder = %home.display(), "Studio could not clear the history folder: {}", err);
            }

            return;
        }

        let path = home.join(FILE);
        let body = serde_json::to_vec(&self.book);

        let Ok(body) = body.inspect_err(|err| warn!(path = %path.display(), "Studio could not encode the history: {}", err)) else {
            return;
        };

        let temp = hidden_temp(&path);
        let written = fs::create_dir_all(&home)
            .and_then(|()| fs::write(&temp, body))
            .and_then(|()| fs::rename(&temp, &path));

        if let Err(err) = written {
            warn!(path = %path.display(), "Studio could not save the history: {}", err);
        }
    }

    fn sweep(&self) {
        let home = self.folder.join(HOME);

        let Ok(found) = fs::read_dir(&home) else {
            return;
        };

        let kept: HashSet<&str> =
            self.book.entries.iter().flat_map(|record| record.files.iter().map(|seat| seat.hash.as_str())).collect();

        if kept.is_empty() {
            if let Err(err) = fs::remove_dir_all(&home) {
                warn!(folder = %home.display(), "Studio could not clear the history folder: {}", err);
            }

            return;
        }

        for entry in found.flatten() {
            let held = entry.file_name();

            if held.to_str().is_none_or(|name| !snapshot(name) || kept.contains(name)) {
                continue;
            }

            if let Err(err) = fs::remove_file(entry.path()) {
                warn!(path = %entry.path().display(), "Studio could not drop a stale history snapshot: {}", err);
            }
        }
    }
}

fn seats(files: &[PathBuf]) -> Option<Vec<Seat>> {
    let mut seated: Vec<Seat> = files
        .iter()
        .map(|path| {
            let name = path.file_name()?.to_str()?.to_owned();
            let body = fs::read(path).ok()?;

            Some(Seat { name, hash: digest(&body), size: body.len() as u64 })
        })
        .collect::<Option<_>>()?;

    seated.sort_by(|a, b| a.name.cmp(&b.name));

    Some(seated)
}

fn snapshot(name: &str) -> bool {
    name.len() == 64 && name.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn digest(body: &[u8]) -> String {
    Sha256::digest(body).iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("bcc-ledger-{}-{}", name, std::process::id()));

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("the scratch folder is made");

        root
    }

    fn blobs(folder: &Path) -> usize {
        fs::read_dir(folder.join(HOME))
            .map_or(0, |found| found.flatten().filter(|entry| entry.file_name().to_str().is_some_and(snapshot)).count())
    }

    #[test]
    fn a_sealed_history_comes_back_and_shares_unchanged_files() {
        let folder = scratch("roundtrip");
        let (sheet, model) = (folder.join("a.png"), folder.join("a.mamodel"));
        let files = vec![sheet.clone(), model.clone()];

        fs::write(&sheet, b"pixels").expect("the sheet is written");
        fs::write(&model, b"one").expect("the model is written");

        let mut ledger = Ledger::open(&folder, &files);

        // Two bulk shots that only differ in the model: the sheet is stored once.
        ledger.push(Kind::Rig, &[(sheet.clone(), b"pixels".to_vec()), (model.clone(), b"one".to_vec())]);
        ledger.push(Kind::Rig, &[(sheet.clone(), b"pixels".to_vec()), (model.clone(), b"two".to_vec())]);
        assert_eq!(blobs(&folder), 3);

        fs::write(&model, b"three").expect("the model is edited");
        ledger.seal(&files);

        let mut reopened = Ledger::open(&folder, &files);
        let (kind, restored) = reopened.pop().expect("the newest shot survives a reopen");

        assert_eq!(kind, Kind::Rig);
        assert!(restored.contains(&(model.clone(), b"two".to_vec())));
        assert_eq!(blobs(&folder), 2, "the popped model snapshot is swept");

        let _ = fs::remove_dir_all(&folder);
    }

    #[test]
    fn the_oldest_entries_go_once_the_snapshots_outgrow_the_budget() {
        let folder = scratch("budget");
        let model = folder.join("a.mamodel");

        let mut ledger = Ledger::open(&folder, std::slice::from_ref(&model));

        for body in [b"aaaaaaaaaa", b"bbbbbbbbbb", b"cccccccccc"] {
            ledger.push(Kind::Model, &[(model.clone(), body.to_vec())]);
        }

        // Three distinct 10-byte snapshots against a 25-byte budget: only the newest two fit.
        assert!(ledger.trim(DEPTH, 25));
        ledger.sweep();

        assert_eq!(ledger.book.entries.len(), 2);
        assert_eq!(blobs(&folder), 2);

        // A lone entry bigger than the budget is still kept, or that edit could never be undone.
        assert!(ledger.trim(DEPTH, 5));
        assert_eq!(ledger.book.entries.len(), 1);

        let _ = fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_file_changed_behind_its_back_drops_the_history() {
        let folder = scratch("drift");
        let model = folder.join("a.mamodel");
        let files = vec![model.clone()];

        fs::write(&model, b"one").expect("the model is written");

        let mut ledger = Ledger::open(&folder, &files);
        ledger.push(Kind::Model, &[(model.clone(), b"one".to_vec())]);
        ledger.seal(&files);

        // Someone edits the model while Studio is closed; undoing would clobber that.
        fs::write(&model, b"outside").expect("the model is edited elsewhere");

        let mut reopened = Ledger::open(&folder, &files);

        assert!(reopened.pop().is_none());
        assert!(!folder.join(HOME).exists(), "nothing is left behind in the set folder");

        let _ = fs::remove_dir_all(&folder);
    }
}
