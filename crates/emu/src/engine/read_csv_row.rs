use super::{AssetStream, read_stream_row};

pub fn read_csv_row(stm: &mut AssetStream<'_>) -> bool {
    read_stream_row(stm, b',')
}
