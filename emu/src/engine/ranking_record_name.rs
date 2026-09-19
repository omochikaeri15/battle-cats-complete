use super::RankingRecord;

pub fn ranking_record_name(record: &RankingRecord) -> Vec<u8> {
    record.name.clone()
}
