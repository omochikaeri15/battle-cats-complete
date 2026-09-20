use super::{AssetStream, Medal, read_cell_stream, read_tsv_row};

pub fn read_medal_name_row(medal: &mut Medal, stm: &mut AssetStream<'_>) {
    if !read_tsv_row(stm) {
        return;
    }

    medal.name = read_cell_stream(stm, 0).to_vec();
    medal.explanation = read_cell_stream(stm, 1).to_vec();
}
