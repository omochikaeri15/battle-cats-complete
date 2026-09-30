use nyanko::chapter::treasure::{TreasureGroup, TreasureText, Treasures, SLOTS};

const GRADES: [&str; 4] = ["None", "Inferior", "Normal", "Superior"];

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub(super) treasures: Treasures,
    names: [Option<TreasureText>; SLOTS],
    sets: [Option<TreasureText>; SLOTS],
    labels: Option<TreasureText>,
    pub grades: [String; 4],
    effects: [Vec<Option<String>>; SLOTS],
}

impl Catalog {
    pub fn new(treasures: Treasures, names: [Option<TreasureText>; SLOTS], sets: [Option<TreasureText>; SLOTS], labels: Option<TreasureText>) -> Self {
        let mut catalog = Self { treasures, names, sets, labels, grades: Default::default(), effects: Default::default() };

        catalog.grades = std::array::from_fn(|level| catalog.grade(level as i32));
        catalog.effects = std::array::from_fn(|slot| {
            (0..catalog.groups(slot).len()).map(|group| text(&catalog.sets, slot).and_then(|sets| sets.set_effect(group))).collect()
        });
        catalog
    }

    pub fn groups(&self, slot: usize) -> &[TreasureGroup] {
        self.treasures.slots.get(slot).and_then(Option::as_ref).map_or(&[], |data| data.groups.as_slice())
    }

    pub fn item(&self, slot: usize, stage: i32) -> Option<&str> {
        text(&self.names, slot)?.name(usize::try_from(stage).ok()?)
    }

    pub fn set(&self, slot: usize, group: usize) -> Option<&str> {
        text(&self.sets, slot)?.name(group)
    }

    pub fn effect(&self, slot: usize, group: usize) -> Option<&str> {
        self.effects.get(slot)?.get(group)?.as_deref()
    }

    fn grade(&self, level: i32) -> String {
        self.labels
            .as_ref()
            .and_then(|labels| labels.grade(level))
            .or_else(|| usize::try_from(level).ok().and_then(|level| GRADES.get(level).copied()))
            .unwrap_or_default()
            .to_owned()
    }
}

fn text(table: &[Option<TreasureText>; SLOTS], slot: usize) -> Option<&TreasureText> {
    table.get(slot)?.as_ref()
}
