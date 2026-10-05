use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use tracing::warn;

use crate::domains::mining;

use super::router::AssetRouter;
use super::{Reader, UniversalTask, get_region_priority, manifest};

const GENERIC: usize = 4;
const HOLLOW: u8 = 0;
const RECOVERED: u8 = 1;
const GENUINE: u8 = 2;
const BASE: u16 = 0;
const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const SPARSE_RATIO: usize = 16;
const SPARSE_FLOOR: usize = 512;
const SET_RIG: char = '-';

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) enum Kind {
    Cat,
    Enemy,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) struct Unit {
    pub kind: Kind,
    pub id: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Home {
    Unit(Unit),
    Set,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Roster {
    Unit(Unit),
    Set(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Role {
    Model,
    Sheet,
    Cuts,
    Motion,
    Image,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Slot {
    pub home: Home,
    pub rig: Option<char>,
    pub role: Role,
}

impl Slot {
    pub(crate) fn read(kind: Kind, folder: &Path, extension: &str) -> Option<Self> {
        let mut parts = folder.components().map(|part| part.as_os_str().to_string_lossy());
        let id: u16 = parts.next().filter(|id| id.len() == 3 && id.bytes().all(|byte| byte.is_ascii_digit()))?.parse().ok()?;
        let rest: Vec<_> = parts.collect();

        let rig = rest.last().is_some_and(|part| part == "anim").then(|| match kind {
            Kind::Cat => rest.first().and_then(|form| form.chars().next()).unwrap_or('f'),
            Kind::Enemy => 'e',
        });

        let role = match (extension, rig.is_some()) {
            ("mamodel", true) => Role::Model,
            ("imgcut", true) => Role::Cuts,
            ("maanim", true) => Role::Motion,
            ("png", true) => Role::Sheet,
            ("png", false) => Role::Image,
            _ => Role::Other,
        };

        Some(Self { home: Home::Unit(Unit { kind, id }), rig, role })
    }

    fn member(extension: &str) -> Option<Self> {
        let role = match extension {
            "mamodel" => Role::Model,
            "imgcut" => Role::Cuts,
            "maanim" => Role::Motion,
            "png" => Role::Sheet,
            _ => return None,
        };

        Some(Self { home: Home::Set, rig: Some(SET_RIG), role })
    }

    fn unit(&self) -> Option<Unit> {
        match self.home {
            Home::Unit(unit) => Some(unit),
            Home::Set => None,
        }
    }
}

fn owner(stem: &str, extension: &str, models: &HashSet<String>, cuts: &HashSet<String>) -> Option<String> {
    match extension {
        "mamodel" | "imgcut" | "png" => (models.contains(stem) || cuts.contains(stem)).then(|| stem.to_string()),
        "maanim" => (1..=stem.len())
            .rev()
            .filter(|&end| stem.is_char_boundary(end) && stem[end..].chars().next().is_none_or(|next| next == '_' || next.is_ascii_digit()))
            .map(|end| &stem[..end])
            .find(|prefix| models.contains(*prefix))
            .map(str::to_string),
        _ => None,
    }
}

type File = (String, Vec<UniversalTask>);

pub(super) type Loose = Vec<File>;

struct Surveyed {
    name: String,
    versions: Vec<UniversalTask>,
    slot: Option<Slot>,
    stem: String,
    extension: String,
}

pub(super) fn seat(files: HashMap<String, Vec<UniversalTask>>, router: &AssetRouter) -> (BTreeMap<Roster, Vec<Member>>, Loose) {
    let surveyed: Vec<Surveyed> = files
        .into_par_iter()
        .map(|(name, versions)| {
            let original = versions.first().map_or(name.as_str(), |task| task.original_name.as_str());
            let slot = router.slot(original);
            let (stem, extension) = router.stem(original);

            Surveyed { name, versions, slot, stem, extension }
        })
        .collect();

    let named = |wanted: &str| -> HashSet<String> {
        surveyed
            .iter()
            .filter(|file| file.slot.is_none() && file.extension == wanted)
            .map(|file| file.stem.clone())
            .collect()
    };

    let (models, cuts) = (named("mamodel"), named("imgcut"));

    let seated: Vec<Result<(Roster, Member), File>> = surveyed
        .into_par_iter()
        .map(|file| {
            let seat = match file.slot {
                Some(slot) => slot.unit().map(|unit| (Roster::Unit(unit), slot)),
                None => owner(&file.stem, &file.extension, &models, &cuts).and_then(|set| Some((Roster::Set(set), Slot::member(&file.extension)?))),
            };

            let Some((roster, slot)) = seat else {
                return Err((file.name, file.versions));
            };

            let destination = file.versions.first().map(|task| router.resolve_destination(&task.original_name, &file.name)).unwrap_or_default();

            Ok((roster, Member { name: file.name, slot, destination, versions: file.versions }))
        })
        .collect();

    let mut rosters: BTreeMap<Roster, Vec<Member>> = BTreeMap::new();
    let mut loose = Vec::new();

    for entry in seated {
        match entry {
            Ok((roster, member)) => rosters.entry(roster).or_default().push(member),
            Err(file) => loose.push(file),
        }
    }

    (rosters, loose)
}

pub(super) struct Member {
    pub name: String,
    pub slot: Slot,
    pub destination: PathBuf,
    pub versions: Vec<UniversalTask>,
}

impl Member {
    fn holds(&self, region: &str) -> bool {
        self.versions.iter().any(|task| task.region_code == region)
    }

    fn newest(&self, region: &str) -> Option<&UniversalTask> {
        self.versions.iter().filter(|task| task.region_code == region).max_by_key(|task| task.chrono_score)
    }

    fn older(&self, region: &str) -> Vec<&UniversalTask> {
        let newest = self.newest(region);

        let mut older: Vec<&UniversalTask> = self
            .versions
            .iter()
            .filter(|task| task.region_code == region && !newest.is_some_and(|newest| std::ptr::eq(newest, *task)))
            .collect();

        older.sort_by_key(|task| std::cmp::Reverse(task.chrono_score));
        older
    }

    fn regions(&self) -> Vec<&str> {
        let mut regions: Vec<&str> = self.versions.iter().map(|task| task.region_code.as_str()).collect();

        regions.sort_unstable();
        regions.dedup();
        regions
    }
}

pub(super) fn stale(members: &[Member], ledger: &manifest::Ledger, hashes: &HashMap<String, HashMap<String, u64>>) -> bool {
    members.iter().any(|member| {
        let Some(held) = ledger.placement(&member.name) else {
            return true;
        };

        held.record.standing.is_none()
            || !member.destination.exists()
            || member.versions.iter().any(|task| {
                if task.is_loose {
                    return task.byte_size > held.record.encrypted;
                }

                let pack = task.pack_name();
                let current = hashes.get(&pack).and_then(|regions| regions.get(&task.region_code)).copied();

                current.is_none() || current != ledger.pack_checksum(&pack, &task.region_code)
            })
    })
}

#[derive(Default)]
struct Spread {
    units: usize,
    cat_base: bool,
    enemy_base: bool,
}

#[derive(Default)]
pub(super) struct Census {
    regions: HashMap<String, HashMap<(usize, u64), Spread>>,
}

impl Census {
    pub(super) fn take(units: &[Vec<Member>], reader: &Reader) -> Self {
        let mut buckets: HashMap<(&str, usize), Vec<(Unit, &UniversalTask)>> = HashMap::new();

        for member in units.iter().flatten() {
            let Some(unit) = member.slot.unit() else {
                continue;
            };

            for region in member.regions() {
                if let Some(task) = member.newest(region) {
                    buckets.entry((region, task.byte_size)).or_default().push((unit, task));
                }
            }
        }

        let crowded: Vec<(&str, Unit, &UniversalTask)> = buckets
            .into_iter()
            .filter(|(_, files)| Self::crowded(files))
            .flat_map(|((region, _), files)| files.into_iter().map(move |(unit, task)| (region, unit, task)))
            .collect();

        let hashed: Vec<(&str, usize, u64, Unit)> = crowded
            .par_iter()
            .filter_map(|&(region, unit, task)| {
                let bytes = reader.read(task).ok()?;

                Some((region, task.byte_size, manifest::hash(&bytes), unit))
            })
            .collect();

        let mut sharers: HashMap<(&str, usize, u64), HashSet<Unit>> = HashMap::new();

        for (region, size, checksum, unit) in hashed {
            sharers.entry((region, size, checksum)).or_default().insert(unit);
        }

        let mut census = Self::default();

        for ((region, size, checksum), units) in sharers {
            if units.len() < 2 {
                continue;
            }

            let spread = Spread {
                units: units.len(),
                cat_base: units.contains(&Unit { kind: Kind::Cat, id: BASE }),
                enemy_base: units.contains(&Unit { kind: Kind::Enemy, id: BASE }),
            };

            census.regions.entry(region.to_string()).or_default().insert((size, checksum), spread);
        }

        census
    }

    fn crowded(files: &[(Unit, &UniversalTask)]) -> bool {
        let units: HashSet<Unit> = files.iter().map(|(unit, _)| *unit).collect();

        units.len() >= GENERIC || units.iter().any(|base| base.id == BASE && units.iter().any(|unit| unit.kind == base.kind && unit.id != BASE))
    }

    fn borrowed(&self, region: &str, unit: Unit, size: usize, checksum: u64) -> bool {
        self.regions.get(region).and_then(|contents| contents.get(&(size, checksum))).is_some_and(|spread| {
            let base = match unit.kind {
                Kind::Cat => spread.cat_base,
                Kind::Enemy => spread.enemy_base,
            };

            spread.units >= GENERIC || (base && unit.id != BASE)
        })
    }
}

fn hollow(bytes: &[u8]) -> bool {
    bytes.split(|&byte| byte == b'\n').nth(2).and_then(|line| std::str::from_utf8(line).ok()).and_then(|line| line.trim().parse::<u32>().ok()) == Some(0)
}

fn blank(bytes: &[u8]) -> bool {
    if bytes.len() < 33 || bytes[..8] != PNG_MAGIC {
        return false;
    }

    let word = |at: usize| bytes.get(at..at + 4).map_or(0, |raw| u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize);
    let (width, height) = (word(16), word(20));

    if width == 1 && height == 1 {
        return true;
    }

    let mut packed = 0usize;
    let mut at = 8usize;

    while at.saturating_add(8) <= bytes.len() {
        let length = word(at);

        if &bytes[at + 4..at + 8] == b"IDAT" {
            packed = packed.saturating_add(length);
        }

        at = at.saturating_add(length).saturating_add(12);
    }

    if packed > width.saturating_mul(height) / SPARSE_RATIO + SPARSE_FLOOR {
        return false;
    }

    image::load_from_memory(bytes).is_ok_and(|decoded| decoded.to_rgba8().pixels().all(|pixel| pixel[3] == 0))
}

struct Copy<'a> {
    task: &'a UniversalTask,
    bytes: Vec<u8>,
    checksum: u64,
    genuine: bool,
}

pub(super) struct Settled {
    pub name: String,
    pub placement: manifest::Placement,
    pub sample: Option<mining::FileDelta>,
    pub written: bool,
}

pub(super) enum Outcome {
    Settled(Box<Settled>),
    Kept,
    Failed,
}

pub(super) struct Bench<'a> {
    pub reader: &'a Reader<'a>,
    pub ledger: &'a manifest::Ledger,
    pub census: &'a Census,
    pub present: &'a HashSet<String>,
    pub tracked: bool,
}

impl Bench<'_> {
    pub(super) fn settle(&self, members: &[Member]) -> Vec<Outcome> {
        let mut desk = Desk { bench: self, members, copies: HashMap::new(), standings: HashMap::new() };
        let mut rigs: BTreeMap<char, Vec<usize>> = BTreeMap::new();

        for (index, member) in members.iter().enumerate() {
            if let Some(rig) = member.slot.rig {
                rigs.entry(rig).or_default().push(index);
            }
        }

        let mut outcomes: Vec<Outcome> = Vec::with_capacity(members.len());

        for (&rig, indexes) in &rigs {
            if desk.outranked(indexes) {
                outcomes.extend(indexes.iter().map(|_| Outcome::Kept));
                continue;
            }

            let chosen = desk.rig(rig, indexes);

            for &index in indexes {
                let pick = chosen.iter().find(|(member, _, _)| *member == index).map(|&(_, region, standing)| (region, standing));

                outcomes.push(pick.map_or(Outcome::Failed, |(region, standing)| desk.finish(index, region, standing)));
            }
        }

        for (index, member) in members.iter().enumerate() {
            if member.slot.rig.is_some() {
                continue;
            }

            if desk.outranked(&[index]) {
                outcomes.push(Outcome::Kept);
                continue;
            }

            let pick = if member.slot.role == Role::Image { desk.image(index) } else { desk.other(index, &rigs) };

            outcomes.push(pick.map_or(Outcome::Failed, |(region, standing)| desk.finish(index, region, standing)));
        }

        outcomes
    }
}

struct Desk<'a, 'b> {
    bench: &'b Bench<'b>,
    members: &'a [Member],
    copies: HashMap<(usize, &'a str), Option<Copy<'a>>>,
    standings: HashMap<(char, &'a str), u8>,
}

impl<'a> Desk<'a, '_> {
    fn open(&self, index: usize, task: &'a UniversalTask) -> Option<Copy<'a>> {
        let member = &self.members[index];

        let bytes = match self.bench.reader.read(task) {
            Ok(bytes) => bytes,
            Err(fault) => {
                warn!("{fault}");
                return None;
            }
        };

        let checksum = manifest::hash(&bytes);

        let placeholder = bytes.is_empty()
            || blank(&bytes)
            || (member.slot.role == Role::Model && hollow(&bytes))
            || member.slot.unit().is_some_and(|unit| self.bench.census.borrowed(&task.region_code, unit, task.byte_size, checksum));

        Some(Copy { task, bytes, checksum, genuine: !placeholder })
    }

    fn load(&mut self, index: usize, region: &'a str) -> Option<&Copy<'a>> {
        if !self.copies.contains_key(&(index, region)) {
            let copy = self.members[index].newest(region).and_then(|task| self.open(index, task));

            self.copies.insert((index, region), copy);
        }

        self.copies.get(&(index, region)).and_then(Option::as_ref)
    }

    fn genuine(&mut self, index: usize, region: &'a str) -> bool {
        self.load(index, region).is_some_and(|copy| copy.genuine)
    }

    fn recover(&self, index: usize, region: &'a str) -> Option<Copy<'a>> {
        self.members[index].older(region).into_iter().find_map(|task| self.open(index, task).filter(|copy| copy.genuine))
    }

    fn regions(&self, indexes: &[usize]) -> Vec<&'a str> {
        let mut regions: Vec<&'a str> = indexes.iter().flat_map(|&index| self.members[index].regions()).collect();

        regions.sort_unstable_by(|left, right| get_region_priority(right).cmp(&get_region_priority(left)).then(left.cmp(right)));
        regions.dedup();
        regions
    }

    fn outranked(&self, indexes: &[usize]) -> bool {
        let contender = self.regions(indexes).first().map_or(0, |region| get_region_priority(region));

        indexes.iter().all(|&index| {
            let member = &self.members[index];

            self.bench.ledger.placement(&member.name).is_some_and(|held| {
                held.record.standing.unwrap_or(GENUINE) == GENUINE
                    && !self.bench.present.contains(&held.record.winner)
                    && get_region_priority(&held.record.winner) > contender
                    && member.destination.exists()
            })
        })
    }

    fn standing(&mut self, rig: char, indexes: &[usize], region: &'a str) -> u8 {
        if let Some(&known) = self.standings.get(&(rig, region)) {
            return known;
        }

        let held: Vec<usize> = indexes.iter().copied().filter(|&index| self.members[index].holds(region)).collect();
        let cast = |role: Role| -> Vec<usize> { held.iter().copied().filter(|&index| self.members[index].slot.role == role).collect() };
        let (models, sheets, cuts) = (cast(Role::Model), cast(Role::Sheet), cast(Role::Cuts));

        let set = indexes.first().is_some_and(|&index| self.members[index].slot.home == Home::Set);
        let complete = set || (!models.is_empty() && (!sheets.is_empty() || cuts.is_empty()));
        let standing = if complete { self.sound(region, &held, &models, &sheets) } else { HOLLOW };

        self.standings.insert((rig, region), standing);
        standing
    }

    fn sound(&mut self, region: &'a str, held: &[usize], models: &[usize], sheets: &[usize]) -> u8 {
        let mut modeled = models.is_empty();
        for &index in models {
            modeled |= self.genuine(index, region);
        }

        let mut sheeted = sheets.is_empty();
        for &index in sheets {
            sheeted |= self.genuine(index, region);
        }

        if modeled && sheeted {
            return GENUINE;
        }

        let suspects = if modeled { sheets } else { held };
        let mut recovered: Vec<(usize, Copy<'a>)> = Vec::new();

        for &index in suspects {
            if !self.genuine(index, region)
                && let Some(copy) = self.recover(index, region)
            {
                recovered.push((index, copy));
            }
        }

        let restored = |wanted: &[usize]| wanted.iter().any(|index| recovered.iter().any(|(found, _)| found == index));

        if !((modeled || restored(models)) && (sheeted || restored(sheets))) {
            return HOLLOW;
        }

        for (index, copy) in recovered {
            self.copies.insert((index, region), Some(copy));
        }

        RECOVERED
    }

    fn rig(&mut self, rig: char, indexes: &[usize]) -> Vec<(usize, &'a str, u8)> {
        let regions = self.regions(indexes);

        for &region in &regions {
            if self.standing(rig, indexes, region) == GENUINE {
                if indexes.iter().all(|&index| self.members[index].holds(region)) {
                    return indexes.iter().map(|&index| (index, region, GENUINE)).collect();
                }

                break;
            }
        }

        let mut order: Vec<(&'a str, u8)> = Vec::with_capacity(regions.len());
        for &region in &regions {
            order.push((region, self.standing(rig, indexes, region)));
        }

        order.sort_by_key(|&(_, standing)| std::cmp::Reverse(standing));

        indexes
            .iter()
            .filter_map(|&index| {
                order.iter().find(|(region, _)| self.members[index].holds(region)).map(|&(region, standing)| (index, region, standing))
            })
            .collect()
    }

    fn image(&mut self, index: usize) -> Option<(&'a str, u8)> {
        let regions = self.regions(&[index]);

        for &region in &regions {
            if self.genuine(index, region) {
                return Some((region, GENUINE));
            }
        }

        for &region in &regions {
            if let Some(copy) = self.recover(index, region) {
                self.copies.insert((index, region), Some(copy));
                return Some((region, RECOVERED));
            }
        }

        regions.first().map(|&region| (region, HOLLOW))
    }

    fn other(&mut self, index: usize, rigs: &BTreeMap<char, Vec<usize>>) -> Option<(&'a str, u8)> {
        let mut best: Option<(&'a str, u8)> = None;

        for region in self.regions(&[index]) {
            let mut backing = HOLLOW;
            for (&rig, indexes) in rigs {
                backing = backing.max(self.standing(rig, indexes, region));
            }

            let standing = if self.genuine(index, region) { backing } else { HOLLOW };

            if standing == GENUINE {
                return Some((region, standing));
            }

            if best.is_none_or(|(_, held)| standing > held) {
                best = Some((region, standing));
            }
        }

        best
    }

    fn finish(&mut self, index: usize, region: &'a str, standing: u8) -> Outcome {
        let members = self.members;
        let member = &members[index];
        let bench = self.bench;
        let held = bench.ledger.placement(&member.name);
        let present = member.destination.exists();

        if let Some(held) = held
            && present
            && held.record.winner != region
            && !bench.present.contains(&held.record.winner)
            && (held.record.standing.unwrap_or(GENUINE), get_region_priority(&held.record.winner)) >= (standing, get_region_priority(region))
        {
            return Outcome::Kept;
        }

        self.load(index, region);

        let Some(copy) = self.copies.remove(&(index, region)).flatten() else {
            return Outcome::Failed;
        };

        let size = copy.bytes.len();

        let settled = match held {
            Some(held) => present && held.record.size == size && held.record.checksum == copy.checksum,
            None => manifest::holds(&member.destination, size, copy.checksum),
        };

        let sample = (bench.tracked && !settled && mining::mineable(&member.name))
            .then(|| {
                let previous = fs::read(&member.destination).ok();
                let from = held.map_or("", |held| held.record.winner.as_str());

                mining::delta(&member.name, from, region, previous.as_deref(), &copy.bytes)
            })
            .flatten();

        if !settled {
            if let Some(parent) = member.destination.parent() {
                let _ = fs::create_dir_all(parent);
            }

            if let Err(error) = fs::write(&member.destination, &copy.bytes) {
                warn!("Could not write {}: {}", member.destination.display(), error);
                return Outcome::Failed;
            }
        }

        Outcome::Settled(Box::new(Settled {
            name: member.name.clone(),
            placement: manifest::Placement {
                pack: copy.task.pack_name(),
                record: manifest::FileRecord {
                    winner: region.to_string(),
                    size,
                    encrypted: copy.task.byte_size,
                    checksum: copy.checksum,
                    standing: Some(standing),
                },
            },
            sample,
            written: !settled,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageFormat, Rgba, RgbaImage};
    use nyanko::pack::cryptology;

    use super::*;
    use crate::domains::settings::ImportStructure;

    fn picture(width: u32, height: u32, alpha: u8, tint: u8) -> Vec<u8> {
        let image = RgbaImage::from_fn(width, height, |x, y| Rgba([tint, (x * 7) as u8, (y * 13) as u8, alpha]));
        let mut encoded = Cursor::new(Vec::new());

        image.write_to(&mut encoded, ImageFormat::Png).expect("png");
        encoded.into_inner()
    }

    struct Stage {
        root: PathBuf,
        sources: Vec<(String, u64, String, Vec<u8>)>,
    }

    impl Stage {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!("bcc-units-{label}"));
            let _ = fs::remove_dir_all(&root);

            Self { root, sources: Vec::new() }
        }

        fn sequel(&self) -> Self {
            Self { root: self.root.clone(), sources: Vec::new() }
        }

        fn ship(&mut self, region: &str, version: u64, name: &str, bytes: Vec<u8>) {
            self.sources.push((region.to_string(), version, name.to_string(), bytes));
        }

        // Every shipped file becomes a loose source on disk, exactly as an unpacked import hands them over.
        fn rosters(&self) -> Vec<Vec<Member>> {
            let router = AssetRouter::new(&self.root.join("game"), ImportStructure::Bcc).expect("router");
            let mut files: HashMap<String, Vec<UniversalTask>> = HashMap::new();

            for (region, version, name, bytes) in &self.sources {
                let source = self.root.join(format!("{region}-{version}"));
                fs::create_dir_all(&source).expect("source dir");
                fs::write(source.join(name), bytes).expect("source file");

                files.entry(name.clone()).or_default().push(UniversalTask {
                    pack_path: source.join(name),
                    original_name: name.clone(),
                    final_name: name.clone(),
                    byte_offset: 0,
                    byte_size: bytes.len(),
                    region_code: region.clone(),
                    chrono_score: *version,
                    is_loose: true,
                });
            }

            let (rosters, loose) = seat(files, &router);

            assert!(loose.is_empty(), "every staged file belongs to a unit or a set");
            rosters.into_values().collect()
        }

        fn import(&self, ledger: &mut manifest::Ledger) -> HashMap<String, String> {
            let rosters = self.rosters();
            let keys = cryptology::Keys::default();
            let reader = Reader { keys: &keys };
            let census = Census::take(&rosters, &reader);
            let present: HashSet<String> = self.sources.iter().map(|(region, ..)| region.clone()).collect();
            let mut winners = HashMap::new();
            let mut placed = Vec::new();

            {
                let bench = Bench { reader: &reader, ledger, census: &census, present: &present, tracked: false };

                for outcome in rosters.iter().flat_map(|members| bench.settle(members)) {
                    if let Outcome::Settled(settled) = outcome {
                        winners.insert(settled.name.clone(), settled.placement.record.winner.clone());
                        placed.push((settled.name, settled.placement));
                    }
                }
            }

            for (name, placement) in placed {
                ledger.place(name, placement);
            }

            winners
        }

        fn rig(&mut self, region: &str, unit: &str, sheet: Vec<u8>, motion: &str) {
            self.ship(region, 0, &format!("{unit}_f.mamodel"), format!("[mamodel]\n3\n1\n{region} {unit} model").into_bytes());
            self.ship(region, 0, &format!("{unit}_f.imgcut"), format!("[imgcut]\n0\n{unit}_f.png\n1\n0,0,8,8,{region}").into_bytes());
            self.ship(region, 0, &format!("{unit}_f.png"), sheet);
            self.ship(region, 0, &format!("{unit}_f00.maanim"), motion.as_bytes().to_vec());
        }
    }

    // The bug this module exists for: a rig must come from one region, even when another
    // region's copy of a single member is larger.
    #[test]
    fn a_rig_is_taken_whole_from_the_first_region_where_it_is_genuine() {
        let mut stage = Stage::new("whole");

        stage.rig("ja", "531", picture(16, 16, 255, 1), "short");
        stage.rig("en", "531", picture(64, 64, 255, 2), "a much longer motion file than the other region ships");

        let winners = stage.import(&mut manifest::Ledger::default());

        assert_eq!(winners.len(), 4);
        assert!(winners.values().all(|winner| winner == "ja"), "{winners:?}");
    }

    #[test]
    fn a_blank_sheet_sends_the_whole_rig_to_the_next_region() {
        let mut stage = Stage::new("blank");

        stage.rig("ja", "120", picture(1, 1, 255, 0), "ja motion");
        stage.rig("en", "120", picture(16, 16, 255, 2), "en motion");
        stage.ship("ja", 0, "uni120_f00.png", picture(16, 16, 0, 0));
        stage.ship("en", 0, "uni120_f00.png", picture(16, 16, 255, 9));

        let winners = stage.import(&mut manifest::Ledger::default());

        assert!(winners.values().all(|winner| winner == "en"), "{winners:?}");
    }

    // Placeholder units are copies of the first unit's art, or one file reused across a batch.
    #[test]
    fn art_borrowed_from_the_base_unit_or_shared_by_a_batch_is_not_genuine() {
        let mut stage = Stage::new("borrowed");
        let base = picture(16, 16, 255, 5);
        let batch = picture(16, 16, 255, 6);

        stage.rig("ja", "000", base.clone(), "base motion");
        stage.rig("ja", "285", base, "copied motion");
        stage.rig("en", "285", picture(16, 16, 255, 7), "real motion");

        for unit in ["601", "602", "603", "604"] {
            stage.rig("ja", unit, batch.clone(), "batch motion");
            stage.rig("en", unit, picture(16, 16, 255, unit.as_bytes()[2]), "real motion");
        }

        let winners = stage.import(&mut manifest::Ledger::default());

        assert_eq!(winners["000_f.png"], "ja");

        for unit in ["285", "601", "602", "603", "604"] {
            assert_eq!(winners[&format!("{unit}_f.png")], "en", "{unit}");
            assert_eq!(winners[&format!("{unit}_f.mamodel")], "en", "{unit}");
        }
    }

    // A unit removed from the game keeps its rig while its images are blanked in a newer pack.
    #[test]
    fn a_blanked_image_falls_back_to_the_newest_version_that_is_not() {
        let mut stage = Stage::new("fallback");
        let original = picture(16, 16, 255, 3);

        stage.rig("ja", "155", original.clone(), "motion");
        stage.ship("ja", 1, "155_f.png", picture(16, 16, 0, 0));
        stage.ship("ja", 0, "udi155_f.png", original.clone());
        stage.ship("ja", 1, "udi155_f.png", picture(16, 16, 0, 0));

        stage.import(&mut manifest::Ledger::default());

        let game = stage.root.join("game").join("cats").join("155").join("f");

        assert_eq!(fs::read(game.join("anim").join("155_f.png")).expect("sheet"), original);
        assert_eq!(fs::read(game.join("udi155_f.png")).expect("banner"), original);

        // An older version is a last resort: another region's current art still beats it.
        let mut rival = stage.sequel();
        rival.sources = stage.sources.clone();
        rival.rig("en", "155", picture(16, 16, 255, 8), "en motion");

        let winners = rival.import(&mut manifest::Ledger::default());

        assert_eq!(winners["155_f.png"], "en");
        assert_eq!(winners["155_f.mamodel"], "en");
    }

    // Menu animations are localized as a whole: the sheet, its cuts and the rigs drawn from it.
    #[test]
    fn an_interface_set_follows_the_same_rule_as_a_unit() {
        let mut stage = Stage::new("interface");

        for (region, tint, motion) in [("ja", 1, "ja"), ("en", 2, "a longer english motion file"), ("th", 3, "the longest motion file of them all, from a language pack")] {
            stage.ship(region, 0, "slot_002.imgcut", format!("[imgcut]\n0\nslot_002.png\n1\n0,0,8,8,{motion}").into_bytes());
            stage.ship(region, 0, "slot_002.png", picture(16, 16, 255, tint));
            stage.ship(region, 0, "gatya_001_get.mamodel", format!("[mamodel]\n3\n1\n{motion}").into_bytes());
            stage.ship(region, 0, "gatya_001_get_000.maanim", motion.as_bytes().to_vec());
        }

        stage.ship("ja", 0, "map023_00.imgcut", b"[imgcut]\n0\nmap023_00.png\n1\n0,0,8,8,ja".to_vec());
        stage.ship("ja", 0, "map023_00.png", picture(16, 16, 0, 0));
        stage.ship("tw", 0, "map023_00.imgcut", b"[imgcut]\n0\nmap023_00.png\n1\n0,0,8,8,tw".to_vec());
        stage.ship("tw", 0, "map023_00.png", picture(16, 16, 255, 4));

        let winners = stage.import(&mut manifest::Ledger::default());

        for name in ["slot_002.imgcut", "slot_002.png", "gatya_001_get.mamodel", "gatya_001_get_000.maanim"] {
            assert_eq!(winners[name], "ja", "{name}");
        }

        assert_eq!(winners["map023_00.png"], "tw");
        assert_eq!(winners["map023_00.imgcut"], "tw");
    }

    // Regions imported one at a time must end where a single import of all of them would.
    #[test]
    fn a_later_import_only_displaces_what_it_outranks() {
        let mut genuine = Stage::new("later-genuine");
        let mut ledger = manifest::Ledger::default();

        genuine.rig("ja", "010", picture(16, 16, 255, 1), "ja motion");
        genuine.import(&mut ledger);

        let mut follow = genuine.sequel();
        follow.rig("en", "010", picture(32, 32, 255, 2), "en motion");

        assert!(follow.import(&mut ledger).is_empty(), "a genuine Japanese rig is not replaced by an English one");

        let mut hollow = Stage::new("later-hollow");
        let mut ledger = manifest::Ledger::default();

        hollow.rig("ja", "011", picture(1, 1, 255, 0), "ja motion");
        hollow.import(&mut ledger);

        let mut rescue = hollow.sequel();
        rescue.rig("en", "011", picture(32, 32, 255, 2), "en motion");

        let winners = rescue.import(&mut ledger);

        assert_eq!(winners.len(), 4);
        assert!(winners.values().all(|winner| winner == "en"), "{winners:?}");
    }
}
