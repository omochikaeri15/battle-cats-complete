use super::AssetStream;

pub fn get_column_count(stm: &AssetStream<'_>) -> u64 {
    stm.cells.len() as u64
}
