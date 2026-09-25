use std::collections::BTreeMap;

use tracing::trace;

use crate::Vfs;

const CANNON_GROWTH: &str = "CC_AllParts_growth.csv";
const STYLE_GROWTH: &str = "CC_DecoParts_growth.csv";
const FOUNDATION_GROWTH: &str = "CC_BaseParts_growth.csv";
const CASTLE_GROWTH: &str = "CC_Castle_growth.csv";
const ID_CELL: usize = 0;
const LEVEL_CELL: usize = 2;
const CASTLE_LEVEL_CELL: usize = 0;
const BARE: i32 = 0;

pub const CANNONS: [&str; 8] = [
    "Cat Cannon",
    "Slow Beam",
    "Iron Wall",
    "Thunderbolt",
    "Waterblast",
    "Holy Blast",
    "Breakerblast",
    "Curseblast",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parts {
    pub cannons: BTreeMap<i32, i32>,
    pub styles: BTreeMap<i32, i32>,
    pub foundations: BTreeMap<i32, i32>,
    pub castle: i32,
}

impl Parts {
    pub fn load(vfs: &Vfs) -> Self {
        trace!("reading the cat base part growth tables");

        let mut styles = growth(vfs, STYLE_GROWTH);
        let mut foundations = growth(vfs, FOUNDATION_GROWTH);

        styles.entry(BARE).or_insert(1);
        foundations.entry(BARE).or_insert(1);

        Self { cannons: growth(vfs, CANNON_GROWTH), styles, foundations, castle: castle(vfs) }
    }
}

pub fn name(part: i32) -> String {
    usize::try_from(part)
        .ok()
        .and_then(|index| CANNONS.get(index))
        .map_or_else(|| format!("Part {part}"), |name| (*name).to_owned())
}

fn rows(vfs: &Vfs, file: &str) -> Option<String> {
    vfs.find(file).and_then(|path| vfs.read(&path).ok()).and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
}

fn castle(vfs: &Vfs) -> i32 {
    rows(vfs, CASTLE_GROWTH).map_or(1, |content| {
        content
            .lines()
            .skip(1)
            .filter_map(|line| line.split(',').nth(CASTLE_LEVEL_CELL).and_then(|cell| cell.trim().parse::<i32>().ok()))
            .fold(1, i32::max)
    })
}

fn growth(vfs: &Vfs, file: &str) -> BTreeMap<i32, i32> {
    let mut highest: BTreeMap<i32, i32> = BTreeMap::new();

    let Some(content) = rows(vfs, file) else {
        return highest;
    };

    for line in content.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        let id = cells.get(ID_CELL).and_then(|cell| cell.trim().parse::<i32>().ok());
        let level = cells.get(LEVEL_CELL).and_then(|cell| cell.trim().parse::<i32>().ok());

        if let (Some(id), Some(level)) = (id, level) {
            let held = highest.entry(id).or_insert(level);

            *held = (*held).max(level);
        }
    }

    highest
}
