use crate::fault::Fault;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct CastleRow {
    pub offset_x: i32,
    pub offset_y: i32,
    pub size: i32,
    pub art_variant: i32,
}

pub fn get_castle_row(castle_vec: &[CastleRow], castle_id: i32) -> Result<CastleRow, Fault> {
    if castle_vec.len() as u32 as i32 <= castle_id {
        return Ok(CastleRow::default());
    }

    castle_vec
        .get(castle_id as i64 as usize)
        .copied()
        .ok_or(Fault::IndexOutOfRange {
            site: "get_castle_row",
            index: castle_id as i64,
            limit: castle_vec.len() as i64,
        })
}
