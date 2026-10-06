use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::common::io::cache::{self, CacheSpec};

use super::payload::Chunk;

const UNKEYED: u64 = 0;
const SLACK: Duration = Duration::from_secs(2);

struct Cache;

impl CacheSpec for Cache {
    type Data = Ledger;
    const FILE: &'static str = "source.bin";
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct Ledger {
    root: String,
    installed: u64,
    chunks: Vec<Held>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(super) struct Held {
    pub(super) name: String,
    pub(super) files: Vec<(String, u64)>,
}

impl Ledger {
    pub(super) fn load(game: &Path) -> Self {
        let root = root(game);

        cache::read::<Cache>().map(|(_, ledger)| ledger).filter(|ledger| !root.is_empty() && ledger.root == root).unwrap_or_default()
    }

    pub(super) fn save(game: &Path, chunks: Vec<Held>) {
        let installed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs());

        cache::write::<Cache>(UNKEYED, &Self { root: root(game), installed, chunks });
    }

    pub(super) fn missing(&self, chunks: &[Chunk]) -> u64 {
        let held: HashSet<&str> = self.chunks.iter().map(|held| held.name.as_str()).collect();

        chunks.iter().filter(|chunk| !held.contains(chunk.name.as_str())).map(|chunk| chunk.size).sum()
    }

    pub(super) fn split<'a>(mut self, game: &Path, chunks: &'a [Chunk]) -> (Vec<Held>, Vec<&'a Chunk>) {
        let latest = UNIX_EPOCH + Duration::from_secs(self.installed) + SLACK;
        let mut held: HashMap<String, Held> = self.chunks.drain(..).map(|held| (held.name.clone(), held)).collect();
        let (mut kept, mut wanted) = (Vec::new(), Vec::new());

        for chunk in chunks {
            match held.remove(&chunk.name).filter(|held| untouched(game, held, latest)) {
                Some(held) => kept.push(held),
                None => wanted.push(chunk),
            }
        }

        (kept, wanted)
    }
}

fn untouched(game: &Path, held: &Held, latest: SystemTime) -> bool {
    held.files.iter().all(|(name, size)| {
        fs::metadata(game.join(name)).is_ok_and(|meta| {
            meta.is_file() && meta.len() == *size && meta.modified().is_ok_and(|modified| modified <= latest)
        })
    })
}

fn root(game: &Path) -> String {
    fs::canonicalize(game).map_or_else(|_| String::new(), |path| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn folder(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("bcc-ledger-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&folder);

        fs::create_dir_all(&folder).unwrap();

        folder
    }

    fn chunk(name: &str, size: u64) -> Chunk {
        serde_json::from_str(&format!(r#"{{"name":"{name}","address":"abc","size":{size},"files":1}}"#)).unwrap()
    }

    fn held(name: &str, files: &[(&str, u64)]) -> Held {
        Held { name: name.to_owned(), files: files.iter().map(|(file, size)| ((*file).to_owned(), *size)).collect() }
    }

    fn ledger(installed: SystemTime, chunks: Vec<Held>) -> Ledger {
        Ledger { root: String::new(), installed: installed.duration_since(UNIX_EPOCH).unwrap().as_secs(), chunks }
    }

    // A chunk is only skipped when every file it supplied is still exactly as the last install left it.
    #[test]
    fn only_chunks_whose_files_are_untouched_are_kept() {
        let game = folder("split");

        for (name, body) in [("a.csv", "12345"), ("b.csv", "123"), ("c.png", "1234567"), ("d.png", "12")] {
            fs::write(game.join(name), body).unwrap();
        }

        let recorded = vec![
            held("same", &[("a.csv", 5), ("b.csv", 3)]),
            held("resized", &[("c.png", 9)]),
            held("lost", &[("d.png", 2), ("gone.png", 4)]),
            held("retired", &[("d.png", 2)]),
        ];
        let published = [chunk("same", 10), chunk("resized", 20), chunk("lost", 30), chunk("fresh", 40)];
        let now = SystemTime::now();

        assert_eq!(ledger(now, recorded.clone()).missing(&published), 40);

        let (kept, wanted) = ledger(now, recorded.clone()).split(&game, &published);

        assert_eq!(kept, vec![recorded[0].clone()]);
        assert_eq!(wanted.iter().map(|chunk| chunk.name.as_str()).collect::<Vec<_>>(), ["resized", "lost", "fresh"]);

        // A file saved after the install was edited by someone, even if its size happens to match.
        let (kept, wanted) = ledger(now - Duration::from_secs(3600), recorded).split(&game, &published);

        assert!(kept.is_empty());
        assert_eq!(wanted.len(), 4);

        let _ = fs::remove_dir_all(&game);
    }

    #[test]
    fn nothing_is_kept_without_a_record() {
        let game = folder("empty");
        let published = [chunk("one", 10), chunk("two", 20)];

        assert_eq!(Ledger::default().missing(&published), 30);
        assert_eq!(Ledger::default().split(&game, &published).1.len(), 2);

        let _ = fs::remove_dir_all(&game);
    }
}
