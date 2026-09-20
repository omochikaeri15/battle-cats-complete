use crate::Fault;

use super::{
    AppContext, AssetStream, TreasureGroup, open_asset_stream, read_csv_cell, read_csv_row,
    string_format_int,
};

pub fn load_treasure_data_csv(ctx: &mut AppContext) -> Result<bool, Fault> {
    let listed = ctx
        .pack_digests
        .get(b"DataLocal.pack".as_slice())
        .is_some_and(|digest| digest.as_slice() == b"7e2daa8236ec08445fa7ddcd7a63f741");

    if !listed {
        return Ok(false);
    }

    let mut index = 0usize;

    while index != 0xa {
        let bytes = if index <= 2 {
            open_asset_stream(ctx, b"treasureData0.csv", 0, 0)?
        } else if index == 3 {
            index += 1;

            continue;
        } else if index <= 6 {
            open_asset_stream(ctx, b"treasureData1.csv", 0, 0)?
        } else {
            let name =
                string_format_int(ctx, b"treasureData2_%d.csv", index.wrapping_sub(7) as i32)?;

            open_asset_stream(ctx, &name, 0, 0)?
        };
        let Some(bytes) = bytes else {
            index += 1;

            continue;
        };
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut groups: Vec<TreasureGroup> = Vec::new();

        while groups.len() != 0xb {
            groups.push(TreasureGroup::default());
        }

        let mut group = 0usize;

        while group != 0xb {
            read_csv_row(stm);

            let value = read_csv_cell(stm, 0) as i32;

            if let Some(record) = groups.get_mut(group) {
                record.count = value;
            }

            group += 1;
        }

        let mut group = 0usize;

        while group != 0xb {
            read_csv_row(stm);

            let mut column = 0i32;

            loop {
                let value = read_csv_cell(stm, column) as i32;

                if let Some(record) = groups.get_mut(group) {
                    record.castles.push(value);
                }

                if groups
                    .get(group)
                    .and_then(|record| record.castles.last())
                    .is_some_and(|last| *last == -1)
                {
                    break;
                }

                if column >= 7 {
                    break;
                }

                column += 1;
            }

            group += 1;
        }

        let mut group = 0usize;

        while group != 0xb {
            read_csv_row(stm);

            let value = read_csv_cell(stm, 0) as i32;

            if let Some(record) = groups.get_mut(group) {
                record.extra = value;
            }

            group += 1;
        }

        let mut group = 0usize;

        while group != 0xb {
            read_csv_row(stm);

            let percent = read_csv_cell(stm, 0) as i32;

            if let Some(record) = groups.get_mut(group) {
                record.percent = percent;
            }

            let effect = read_csv_cell(stm, 1) as i32;

            if let Some(record) = groups.get_mut(group) {
                record.effect = effect;
            }

            let flag = read_csv_cell(stm, 2) as i32;

            if let Some(record) = groups.get_mut(group) {
                record.chapter_only = u8::from(flag != 0);
            }

            group += 1;
        }

        if let Some(chapter) = ctx.treasure_store.get_mut(index) {
            *chapter = groups;
        }

        index += 1;
    }

    Ok(true)
}
