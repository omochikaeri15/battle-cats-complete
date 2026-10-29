#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct RankingRecord {
    pub id: i32,
    pub status: i32,
    pub score: i32,
    pub rank: i32,
    pub name: Vec<u8>,
}

pub fn ranking_record_id(record: &RankingRecord) -> i32 {
    record.id
}
