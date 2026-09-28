use crate::Fault;

use super::{
    AppContext, AssetStream, get_column_count, open_asset_stream, read_cell_stream, read_csv_cell,
    read_tsv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct GatyaChanceAnimation {
    pub pair: [i32; 2],
    pub triples: Vec<[i32; 3]>,
}

pub fn load_gatya_chance_animation_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.gatya_chance_animations.clear();

    let Some(bytes) = open_asset_stream(ctx, b"GatyaData_Option_ChanceAnimation.tsv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_tsv_row(stm);

    loop {
        if !read_tsv_row(stm) || (get_column_count(stm) as i64) < 2 {
            break;
        }

        if read_cell_stream(stm, 0).is_empty() {
            break;
        }

        ctx.gatya_chance_animations.push(GatyaChanceAnimation::default());

        let first = read_csv_cell(stm, 0) as i32;

        if let Some(last) = ctx.gatya_chance_animations.last_mut() {
            last.pair[0] = first;
        }

        let second = read_csv_cell(stm, 1) as i32;

        if let Some(last) = ctx.gatya_chance_animations.last_mut() {
            last.pair[1] = second;
        }

        let mut column = 4i32;

        while (column as i64) < get_column_count(stm) as i64 {
            let triple = [
                read_csv_cell(stm, column.wrapping_sub(2)) as i32,
                read_csv_cell(stm, column.wrapping_sub(1)) as i32,
                read_csv_cell(stm, column) as i32,
            ];

            if let Some(last) = ctx.gatya_chance_animations.last_mut() {
                last.triples.push(triple);
            }

            column = column.wrapping_add(3);
        }
    }

    Ok(())
}
