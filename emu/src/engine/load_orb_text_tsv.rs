use crate::Fault;

use super::{AppContext, AssetStream, get_column_count, open_asset_stream, read_cell_stream, read_tsv_row};

pub fn load_orb_text_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.orb_explanations.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipment_explonation.tsv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_tsv_row(stm) {
        if (get_column_count(stm) as i32) < 2 {
            return Ok(());
        }

        let name = read_cell_stream(stm, 0).to_vec();
        let explanation = read_cell_stream(stm, 1).to_vec();

        ctx.orb_store.orb_explanations.push([name, explanation]);
    }

    Ok(())
}
