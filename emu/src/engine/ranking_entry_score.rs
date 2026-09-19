use super::{ranking_record_score, RankingRecord};

pub fn ranking_entry_score(entry: &Option<RankingRecord>) -> i32 {
    entry.as_ref().map_or(0, ranking_record_score)
}
