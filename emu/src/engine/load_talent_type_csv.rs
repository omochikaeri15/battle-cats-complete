use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

const SITE: &str = "load_talent_type_csv";

pub fn load_talent_type_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"SkillAcquisition.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            return Ok(());
        }

        let unit_id = read_csv_cell(stm, 0) as i32;
        let type_id = read_csv_cell(stm, 1) as i32;
        let row = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
        let Some(slot) = row.first_mut() else {
            return Err(Fault::NullPointer { site: SITE });
        };

        *slot = type_id;

        let mut group = 0i32;

        while group != 8 {
            let base = group.wrapping_mul(14);
            let row = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

            if base.wrapping_add(0xe) >= get_column_count(stm) as i32
                || !cell_is_int(stm, base.wrapping_add(2))
            {
                let Some(slot) = row.get_mut(1i32.wrapping_add(base) as usize) else {
                    return Err(Fault::NullPointer { site: SITE });
                };

                *slot = 0;

                break;
            }

            let abil = read_csv_cell(stm, base.wrapping_add(2)) as i32;
            let max_level = read_csv_cell(stm, base.wrapping_add(3)) as i32;
            let Some(window) = row.get_mut(base as usize..base.wrapping_add(15) as usize) else {
                return Err(Fault::NullPointer { site: SITE });
            };
            let Some(slot) = window.get_mut(1) else {
                return Err(Fault::NullPointer { site: SITE });
            };

            *slot = abil;

            let Some(slot) = window.get_mut(2) else {
                return Err(Fault::NullPointer { site: SITE });
            };

            *slot = max_level;

            if *slot == 0 {
                *slot = 1;
            }

            let mut pair = 0i32;

            while pair != 4 {
                let low = read_csv_cell(stm, base.wrapping_add(4).wrapping_add(pair * 2)) as i32;
                let Some(slot) = window.get_mut(3i32.wrapping_add(pair) as usize) else {
                    return Err(Fault::NullPointer { site: SITE });
                };

                *slot = low;

                let high = read_csv_cell(stm, base.wrapping_add(5).wrapping_add(pair * 2)) as i32;
                let Some(slot) = window.get_mut(7i32.wrapping_add(pair) as usize) else {
                    return Err(Fault::NullPointer { site: SITE });
                };

                *slot = high;
                pair += 1;
            }

            for (offset, column) in [(11usize, 0xci32), (12, 0xd), (13, 0xe), (14, 0xf)] {
                let value = read_csv_cell(stm, base.wrapping_add(column)) as i32;
                let Some(slot) = window.get_mut(offset) else {
                    return Err(Fault::NullPointer { site: SITE });
                };

                *slot = value;
            }

            group += 1;
        }

        let mut kept: BTreeMap<i32, i32> = BTreeMap::new();
        let mut group = 0i32;

        while group != 8 {
            let index = 1i32.wrapping_add(group.wrapping_mul(14)) as usize;
            let abil = ctx
                .talent_definitions
                .entry(unit_id)
                .or_insert([0; 0x71])
                .get(index)
                .copied()
                .unwrap_or(0);

            kept.insert(abil, 0);

            if !ctx.talent_levels.is_empty()
                && ctx
                    .talent_levels
                    .entry(unit_id)
                    .or_default()
                    .range(abil..)
                    .next()
                    .is_some()
            {
                let level = *ctx
                    .talent_levels
                    .entry(unit_id)
                    .or_default()
                    .entry(abil)
                    .or_default();

                kept.insert(abil, level);
            }

            group += 1;
        }

        *ctx.talent_levels.entry(unit_id).or_default() = kept;
    }

    Ok(())
}
