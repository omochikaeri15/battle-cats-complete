use std::collections::HashMap;

use nyanko::cat::unit::{Equipment, NyancomboData};
use serde::{Deserialize, Serialize};

use crate::common::architecture;
use crate::Vfs;

use super::orb;

pub const LINEUP_SLOTS: usize = 10;
pub const BENCH_SLOTS: usize = 5;
pub const TOP_ROW: usize = 5;

const FIRST_TALENT_FORM: usize = 2;
const UNNAMED: &str = "New Lineup";
const MOD_HISTORIES: usize = 3;
const MOUNT_JOIN: &str = "+";

pub fn mount_of(vfs: &Vfs) -> String {
    let mods: Vec<Box<str>> = vfs.mounted().into_iter().filter(|mount| !mount.eq_ignore_ascii_case(architecture::GAME)).collect();

    if mods.is_empty() {
        return architecture::GAME.to_owned();
    }

    mods.join(MOUNT_JOIN)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Member {
    pub id: u32,
    pub form: usize,
    pub level: String,
    pub talents: HashMap<u8, u8>,
    pub orbs: Vec<Option<u32>>,
}

impl Member {
    pub fn levels(&self) -> (u32, u32) {
        let mut terms = self.level.split('+').map(|term| term.trim().parse::<u32>().unwrap_or(0));
        let base = terms.next().unwrap_or(0).max(1);

        (base, terms.sum())
    }

    pub fn talented(&self) -> bool {
        self.form >= FIRST_TALENT_FORM && self.talents.values().any(|level| *level > 0)
    }

    pub fn equipped(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        self.orbs
            .iter()
            .enumerate()
            .filter(|_| self.form >= FIRST_TALENT_FORM)
            .filter_map(|(slot, orb)| orb.map(|orb| (slot, orb)))
    }

    pub fn worn<'a>(&'a self, slots: usize, orbs: &'a [Equipment]) -> impl Iterator<Item = (usize, u32)> + 'a {
        self.equipped().filter(move |(slot, held)| *slot < slots && orb::kind(orbs, *held).is_some())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    Slot(usize),
    Bench(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lineup {
    pub name: String,
    pub slots: Vec<Member>,
    pub bench: [Option<Member>; BENCH_SLOTS],
}

impl Default for Lineup {
    fn default() -> Self {
        Self { name: UNNAMED.to_owned(), slots: Vec::new(), bench: Default::default() }
    }
}

impl Lineup {
    pub fn get(&self, cell: Cell) -> Option<&Member> {
        match cell {
            Cell::Slot(index) => self.slots.get(index),
            Cell::Bench(index) => self.bench.get(index).and_then(Option::as_ref),
        }
    }

    pub fn get_mut(&mut self, cell: Cell) -> Option<&mut Member> {
        match cell {
            Cell::Slot(index) => self.slots.get_mut(index),
            Cell::Bench(index) => self.bench.get_mut(index).and_then(Option::as_mut),
        }
    }

    pub fn find(&self, id: u32) -> Option<Cell> {
        self.slots.iter().position(|member| member.id == id).map(Cell::Slot).or_else(|| {
            self.bench.iter().position(|held| held.as_ref().is_some_and(|member| member.id == id)).map(Cell::Bench)
        })
    }

    pub fn add(&mut self, member: Member) -> Option<Cell> {
        if let Some(held) = self.find(member.id) {
            return Some(held);
        }

        if self.slots.len() < LINEUP_SLOTS {
            self.slots.push(member);

            return Some(Cell::Slot(self.slots.len() - 1));
        }

        let open = self.bench.iter().position(Option::is_none)?;

        self.bench[open] = Some(member);

        Some(Cell::Bench(open))
    }

    pub fn take(&mut self, cell: Cell) -> Option<Member> {
        match cell {
            Cell::Slot(index) => (index < self.slots.len()).then(|| self.slots.remove(index)),
            Cell::Bench(index) => self.bench.get_mut(index).and_then(Option::take),
        }
    }

    pub fn place(&mut self, member: Member, target: Cell) -> Option<Cell> {
        if let Some(held) = self.find(member.id) {
            self.take(held);
        }

        match target {
            Cell::Slot(index) if index < self.slots.len() => {
                let displaced = std::mem::replace(&mut self.slots[index], member);

                self.shelve(displaced);

                Some(Cell::Slot(index))
            }
            Cell::Slot(_) if self.slots.len() < LINEUP_SLOTS => {
                self.slots.push(member);

                Some(Cell::Slot(self.slots.len() - 1))
            }
            Cell::Slot(_) => None,
            Cell::Bench(index) => {
                let held = self.bench.get_mut(index)?;
                let displaced = held.replace(member);

                if let Some(displaced) = displaced {
                    self.shelve(displaced);
                }

                Some(Cell::Bench(index))
            }
        }
    }

    pub fn shift(&mut self, from: Cell, target: Cell) {
        if from == target {
            return;
        }

        let Some(member) = self.take(from) else {
            return;
        };

        let landed = match (from, target) {
            (Cell::Slot(_), Cell::Slot(index)) => {
                let at = index.min(self.slots.len());

                self.slots.insert(at, member);

                return;
            }
            _ => self.place(member, target),
        };

        if landed.is_none() {
            tracing::debug!("a lineup move had nowhere to land");
        }
    }

    fn shelve(&mut self, member: Member) {
        if let Some(open) = self.bench.iter().position(Option::is_none) {
            self.bench[open] = Some(member);
        }
    }

    pub fn active(&self, row: &NyancomboData) -> bool {
        row.is_active()
            && row.members().all(|slot| {
                self.slots.iter().take(TOP_ROW).any(|member| {
                    i32::try_from(member.id) == Ok(slot.unit_id) && i32::try_from(member.form).is_ok_and(|form| form >= slot.form)
                })
            })
    }

    pub fn adopt(&mut self, row: &NyancomboData, recruit: impl Fn(u32, usize) -> Member) {
        let wanted: Vec<(u32, usize)> = row
            .members()
            .filter_map(|slot| Some((u32::try_from(slot.unit_id).ok()?, usize::try_from(slot.form).ok()?)))
            .collect();

        if wanted.is_empty() {
            return;
        }

        let fielded = wanted.iter().filter(|(id, _)| self.slots.iter().any(|member| member.id == *id)).count();
        let missing = wanted.len() - fielded;

        if self.slots.len() + missing > LINEUP_SLOTS {
            self.shelve_combo(&wanted, recruit);

            return;
        }

        let mut front: Vec<Member> = Vec::with_capacity(wanted.len());

        for (id, form) in wanted {
            let held = self.slots.iter().position(|member| member.id == id).map(|index| self.slots.remove(index));

            if let Some(Cell::Bench(index)) = self.find(id) {
                self.bench[index] = None;
            }

            let mut member = held.unwrap_or_else(|| recruit(id, form));

            member.form = member.form.max(form);
            front.push(member);
        }

        front.append(&mut self.slots);
        self.slots = front;
    }

    fn shelve_combo(&mut self, wanted: &[(u32, usize)], recruit: impl Fn(u32, usize) -> Member) {
        let mut overridden = 0usize;

        for (id, form) in wanted {
            if let Some(member) = self.slots.iter_mut().find(|member| member.id == *id) {
                member.form = member.form.max(*form);

                continue;
            }

            let held = self.bench.iter().position(|held| held.as_ref().is_some_and(|member| member.id == *id));
            let open = held.or_else(|| self.bench.iter().position(Option::is_none));

            let index = open.unwrap_or_else(|| {
                let index = overridden % BENCH_SLOTS;

                overridden += 1;
                index
            });

            self.bench[index] = Some(recruit(*id, *form));
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Loadout {
    pub form: usize,
    pub level: String,
    pub talents: HashMap<u8, u8>,
    pub orbs: Vec<Option<u32>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub mount: String,
    pub units: HashMap<u32, Loadout>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Roster {
    pub lineups: Vec<Lineup>,
    pub selected: usize,
    pub histories: Vec<History>,
}

impl Default for Roster {
    fn default() -> Self {
        Self { lineups: vec![Lineup::default()], selected: 0, histories: Vec::new() }
    }
}

impl Roster {
    pub fn current(&self) -> Option<&Lineup> {
        self.lineups.get(self.selected)
    }

    fn history(&self, mount: &str) -> Option<&History> {
        self.histories.iter().find(|history| history.mount == mount)
    }

    pub fn remember(&mut self, mount: &str) {
        let Some(lineup) = self.lineups.get(self.selected) else {
            return;
        };
        let seen: Vec<(u32, Loadout)> = lineup
            .slots
            .iter()
            .chain(lineup.bench.iter().flatten())
            .map(|member| {
                let loadout = Loadout {
                    form: member.form,
                    level: member.level.clone(),
                    talents: member.talents.clone(),
                    orbs: member.orbs.clone(),
                };

                (member.id, loadout)
            })
            .collect();

        let mut history = self
            .histories
            .iter()
            .position(|history| history.mount == mount)
            .map_or_else(|| History { mount: mount.to_owned(), units: HashMap::new() }, |at| self.histories.remove(at));

        history.units.extend(seen);
        self.histories.insert(0, history);

        let mut mods = 0;

        self.histories.retain(|history| {
            if history.mount == architecture::GAME {
                return true;
            }

            mods += 1;
            mods <= MOD_HISTORIES
        });
    }

    pub fn recall(&self, id: u32, mount: &str) -> Option<&Loadout> {
        self.history(mount)?.units.get(&id)
    }

    pub fn dress(&self, mut member: Member, least: Option<usize>, mount: &str) -> Member {
        let Some(past) = self.recall(member.id, mount) else {
            return member;
        };

        member.form = least.map_or(past.form, |wanted| wanted.max(past.form));
        member.level = past.level.clone();
        member.talents = past.talents.clone();
        member.orbs = past.orbs.clone();
        member
    }

    pub fn current_mut(&mut self) -> &mut Lineup {
        if self.lineups.is_empty() {
            self.lineups.push(Lineup::default());
        }

        self.selected = self.selected.min(self.lineups.len() - 1);

        &mut self.lineups[self.selected]
    }

    pub fn create(&mut self) {
        self.lineups.push(Lineup::default());
        self.selected = self.lineups.len() - 1;
    }

    pub fn delete(&mut self) {
        if self.selected < self.lineups.len() {
            self.lineups.remove(self.selected);
        }

        if self.lineups.is_empty() {
            self.lineups.push(Lineup::default());
        }

        self.selected = self.selected.min(self.lineups.len() - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAME: &str = architecture::GAME;

    fn unit(id: u32) -> Member {
        Member { id, level: "30".to_owned(), ..Member::default() }
    }

    #[test]
    fn a_units_loadout_comes_back_when_it_is_recruited_again() {
        let mut roster = Roster::default();
        let mut dressed = unit(7);

        dressed.form = 2;
        dressed.level = "50+10".to_owned();
        dressed.talents.insert(3, 6);
        dressed.orbs = vec![Some(11), None];

        roster.current_mut().add(dressed);
        roster.remember(GAME);

        // Dropping the unit must not forget it - that is the whole point.
        roster.current_mut().slots.clear();
        roster.remember(GAME);

        let past = roster.recall(7, GAME).expect("unit 7 remembered");

        assert_eq!(past.form, 2);
        assert_eq!(past.level, "50+10");
        assert_eq!(past.talents.get(&3), Some(&6));
        assert_eq!(past.orbs, vec![Some(11), None]);
        assert!(roster.recall(8, GAME).is_none());
    }

    #[test]
    fn a_combo_form_is_a_floor_not_an_override() {
        let mut roster = Roster::default();
        let mut dressed = unit(7);

        dressed.form = 2;
        dressed.talents.insert(1, 4);
        roster.current_mut().add(dressed);
        roster.remember(GAME);

        // The combo wants a lower form than the one remembered, so the
        // remembered one wins and the kit still comes back.
        let kept = roster.dress(unit(7), Some(1), GAME);

        assert_eq!(kept.form, 2);
        assert_eq!(kept.talents.get(&1), Some(&4));

        // Below what the combo needs, the form is bumped up to meet it.
        let bumped = roster.dress(unit(7), Some(3), GAME);

        assert_eq!(bumped.form, 3);
        assert_eq!(bumped.talents.get(&1), Some(&4));

        // With nothing remembered the member is left exactly as built.
        assert_eq!(roster.dress(unit(9), Some(1), GAME).form, 0);
    }

    #[test]
    fn a_mod_keeps_its_own_history_and_only_three_mods_are_kept() {
        let mut roster = Roster::default();
        let mut dressed = unit(7);

        dressed.level = "50+10".to_owned();
        roster.current_mut().add(dressed);
        roster.remember("modded");

        // A kit built under a mod never leaks into the game's history.
        assert!(roster.recall(7, GAME).is_none());
        assert_eq!(roster.recall(7, "modded").map(|past| past.level.as_str()), Some("50+10"));

        roster.remember(GAME);

        for mount in ["second", "third", "fourth"] {
            roster.remember(mount);
        }

        // The oldest mod falls off, the game never does.
        assert!(roster.recall(7, "modded").is_none());
        assert!(roster.recall(7, GAME).is_some());
        assert!(roster.recall(7, "second").is_some());
    }

    fn combo(members: &[(i32, i32)]) -> NyancomboData {
        let mut cells = vec!["0".to_owned(), "1".to_owned(), "-1".to_owned()];

        for slot in 0..5 {
            let (id, form) = members.get(slot).copied().unwrap_or((-1, -1));

            cells.push(id.to_string());
            cells.push(form.to_string());
        }

        cells.extend(["0".to_owned(), "0".to_owned(), "0".to_owned()]);

        NyancomboData::parse(cells.join(","), None).unwrap().remove(0)
    }

    // The game never leaves a hole in the deck, so removing a middle unit closes the gap.
    #[test]
    fn a_removed_slot_closes_up() {
        let mut lineup = Lineup::default();

        for id in 0..4 {
            lineup.add(unit(id));
        }

        lineup.take(Cell::Slot(1));

        assert_eq!(lineup.slots.iter().map(|member| member.id).collect::<Vec<_>>(), [0, 2, 3]);
    }

    #[test]
    fn the_eleventh_unit_overflows_to_the_bench() {
        let mut lineup = Lineup::default();

        for id in 0..10 {
            lineup.add(unit(id));
        }

        assert_eq!(lineup.add(unit(10)), Some(Cell::Bench(0)));
        assert_eq!(lineup.add(unit(3)), Some(Cell::Slot(3)), "a unit already fielded is found, not doubled");
    }

    #[test]
    fn the_bench_keeps_its_holes() {
        let mut lineup = Lineup::default();

        lineup.place(unit(7), Cell::Bench(3));

        assert!(lineup.bench[0].is_none());
        assert_eq!(lineup.bench[3].as_ref().map(|member| member.id), Some(7));
    }

    #[test]
    fn a_combo_needs_the_form_it_names() {
        let mut lineup = Lineup::default();
        let row = combo(&[(5, 1), (6, 0)]);

        lineup.add(unit(5));
        lineup.add(unit(6));
        assert!(!lineup.active(&row));

        lineup.slots[0].form = 2;
        assert!(lineup.active(&row));
    }

    #[test]
    fn adopting_a_combo_fills_open_slots_and_raises_forms() {
        let mut lineup = Lineup::default();
        let row = combo(&[(5, 1), (6, 0)]);

        lineup.add(unit(5));
        lineup.adopt(&row, |id, form| Member { form, ..unit(id) });

        assert!(lineup.active(&row));
        assert_eq!(lineup.slots.len(), 2);
        assert_eq!(lineup.slots[0].form, 1);
    }

    // With the deck full the combo lands on the bench, and with the bench full too it
    // takes over only as many bench cells as it needs.
    #[test]
    fn a_combo_with_nowhere_to_go_overrides_the_bench() {
        let mut lineup = Lineup::default();

        for id in 0..10 {
            lineup.add(unit(id));
        }

        for index in 0..BENCH_SLOTS {
            lineup.place(unit(100 + index as u32), Cell::Bench(index));
        }

        lineup.adopt(&combo(&[(50, 0), (51, 0)]), |id, form| Member { form, ..unit(id) });

        let bench: Vec<u32> = lineup.bench.iter().flatten().map(|member| member.id).collect();

        assert_eq!(bench, [50, 51, 102, 103, 104]);
        assert_eq!(lineup.slots.len(), 10);
    }

    // The game only counts a combo whose units all sit on the top row, so a combo has to
    // be pushed in from the first slot rather than appended behind a full row.
    #[test]
    fn a_combo_lands_on_the_top_row() {
        let mut lineup = Lineup::default();
        let row = combo(&[(50, 0), (3, 0)]);

        for id in 0..7 {
            lineup.add(unit(id));
        }

        assert!(!lineup.active(&combo(&[(6, 0)])), "a unit on the second row powers nothing");

        lineup.adopt(&row, |id, form| Member { form, ..unit(id) });

        assert_eq!(lineup.slots.iter().map(|member| member.id).collect::<Vec<_>>(), [50, 3, 0, 1, 2, 4, 5, 6]);
        assert!(lineup.active(&row));
    }

    #[test]
    fn a_level_with_a_plus_splits_in_two() {
        assert_eq!(Member { level: "50+30".to_owned(), ..Member::default() }.levels(), (50, 30));
        assert_eq!(Member { level: String::new(), ..Member::default() }.levels(), (1, 0));
    }
}
