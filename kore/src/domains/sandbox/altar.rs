use std::collections::BTreeMap;
use std::fs;

use nyanko::chapter::map::{DemonCastleDefine, DemonCastleLimit};

use crate::Vfs;

const LIMIT_FILE: &str = "DemonCastlelimit.csv";
const DEFINE_FILE: &str = "DemonCastledefine.csv";
const CASTLE_ROW_SHIFT: i32 = 2;

#[derive(Clone, Debug, Default)]
pub struct Altars {
    limits: BTreeMap<i32, DemonCastleLimit>,
    castles: Vec<i32>,
}

impl Altars {
    pub fn load(vfs: &Vfs) -> Self {
        let read = |name: &str| vfs.locate(name).and_then(|path| fs::read(path).ok());
        let limits = read(LIMIT_FILE)
            .and_then(|bytes| DemonCastleLimit::parse(bytes, None).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|row| (row.stage_key, row))
            .collect();
        let castles = read(DEFINE_FILE)
            .and_then(|bytes| DemonCastleDefine::parse(bytes, None).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|row| row.enemy_id)
            .collect();

        Self { limits, castles }
    }

    pub fn level_cap(&self, castle_row: i32, level: Option<i32>) -> Option<u32> {
        let enemy = castle_row - CASTLE_ROW_SHIFT;

        if !self.castles.contains(&enemy) {
            return None;
        }

        let mut budget = level.map_or(0, |level| level.saturating_sub(1));
        let mut cap = 0i32;
        let mut unsealed = false;

        for row in self.limits.values() {
            let cleared = level.is_none() || (row.unseal == 0 && row.amount <= budget);

            if cleared && level.is_some() {
                budget -= row.amount;
            }

            if cleared && row.enemy_id == enemy {
                cap = cap.wrapping_add(row.amount);
                unsealed |= row.unseal != 0;
            }
        }

        (!unsealed).then(|| u32::try_from(cap).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn altars(limits: &str, defines: &str) -> Altars {
        Altars {
            limits: DemonCastleLimit::parse(limits, None).unwrap().into_iter().map(|row| (row.stage_key, row)).collect(),
            castles: DemonCastleDefine::parse(defines, None).unwrap().into_iter().map(|row| row.enemy_id).collect(),
        }
    }

    #[test]
    fn the_budget_is_spent_in_stage_order_and_the_unseal_row_is_never_bought() {
        let table = altars("100,1,0,7\n101,1,0,7\n102,1,0,7\n200,0,1,7\n", "7,8\n");

        assert_eq!(table.level_cap(9, Some(3)), Some(2), "level 3 buys two rows");
        assert_eq!(table.level_cap(9, Some(10)), Some(3), "the cap stops at the rows there are");
        assert_eq!(table.level_cap(9, None), None, "no level means every row is cleared, the unseal row included");
        assert_eq!(table.level_cap(11, Some(3)), None, "a castle that is not an altar is never sealed");
    }
}
