use std::ops::RangeInclusive;

use crate::Vfs;

pub const FILE: &str = "stage_conditions.csv";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Draw {
    map: i32,
    slots: RangeInclusive<i32>,
    layouts: RangeInclusive<i32>,
}

impl Draw {
    fn positional(&self) -> bool {
        self.slots.end().wrapping_sub(*self.slots.start())
            == self.layouts.end().wrapping_sub(*self.layouts.start())
    }

    fn slot_of(&self, layout: i32) -> i32 {
        if self.positional() {
            self.slots.start().wrapping_add(layout.wrapping_sub(*self.layouts.start()))
        } else {
            *self.slots.start()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Lottery {
    draws: Vec<Draw>,
}

impl Lottery {
    pub fn load(vfs: &Vfs) -> Self {
        vfs.load(FILE).map_or_else(Self::default, |bytes| Self::parse(&bytes))
    }

    pub fn parse(bytes: &[u8]) -> Self {
        let text = String::from_utf8_lossy(bytes);
        let draws = text
            .lines()
            .filter_map(|line| {
                let mut cells = line.split(',').map(|cell| cell.trim().parse::<i32>().ok());
                let map = cells.next()??;
                let first_slot = cells.next()??;
                let last_slot = cells.next()??;
                let first_layout = cells.next()??;
                let last_layout = cells.next()??;

                (first_slot <= last_slot && first_layout <= last_layout).then_some(Draw {
                    map,
                    slots: first_slot..=last_slot,
                    layouts: first_layout..=last_layout,
                })
            })
            .collect();

        Self { draws }
    }

    pub fn slot_for(&self, map: i32, layout: i32) -> Option<i32> {
        self.draws
            .iter()
            .filter(|draw| draw.map == map && draw.layouts.contains(&layout))
            .map(|draw| draw.slot_of(layout))
            .min()
    }

    pub fn drawn_by(&self, map: i32, slot: i32) -> Option<RangeInclusive<i32>> {
        self.draws
            .iter()
            .find(|draw| draw.map == map && draw.slots.contains(&slot))
            .map(|draw| draw.layouts.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The shipped table's own header names the columns: map id, stage number
    // start/end, then the lottery source data id start/end.
    const TABLE: &[u8] = b"\
33000,0,8,0,8,
33000,9,9,90,93, //10th floor boss
33000,10,18,9,26,
33000,19,19,94,97,
33000,99,99,112,112,
//map,stage start,stage end,source start,source end
";

    #[test]
    fn equal_length_ranges_pair_positionally() {
        let lottery = Lottery::parse(TABLE);

        // Nine slots drawing from nine layouts is one layout per slot, so the
        // fifth layout belongs to the fifth floor, not the first.
        assert_eq!(lottery.slot_for(33000, 5), Some(5));
        assert_eq!(lottery.slot_for(33000, 112), Some(99));
    }

    #[test]
    fn pools_fall_back_to_the_first_slot_that_can_draw_them() {
        let lottery = Lottery::parse(TABLE);

        assert_eq!(lottery.slot_for(33000, 91), Some(9));
        assert_eq!(lottery.slot_for(33000, 20), Some(10));
        assert_eq!(lottery.slot_for(33000, 95), Some(19));
    }

    #[test]
    fn unknown_maps_and_layouts_have_no_slot() {
        let lottery = Lottery::parse(TABLE);

        assert_eq!(lottery.slot_for(24000, 5), None);
        assert_eq!(lottery.slot_for(33000, 500), None);
        assert_eq!(Lottery::default().slot_for(33000, 5), None);
    }

    #[test]
    fn comment_and_header_lines_are_skipped() {
        assert_eq!(Lottery::parse(TABLE).draws.len(), 5);
    }
}
