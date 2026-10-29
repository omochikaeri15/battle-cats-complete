use super::{RankingRecord, ranking_record_id};

pub fn ranking_entry_id(entry: &Option<RankingRecord>) -> i32 {
    entry.as_ref().map_or(0, ranking_record_id)
}
