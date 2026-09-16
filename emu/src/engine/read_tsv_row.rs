use super::{read_stream_row, AssetStream};

pub fn read_tsv_row(stm: &mut AssetStream<'_>) -> bool {
    read_stream_row(stm, b'\t')
}
