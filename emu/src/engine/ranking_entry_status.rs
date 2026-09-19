use super::{ranking_record_status, RankingRecord};

pub fn ranking_entry_status(entry: &Option<RankingRecord>) -> i32 {
    entry.as_ref().map_or(0, ranking_record_status)
}
