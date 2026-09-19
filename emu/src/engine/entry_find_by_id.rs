use super::{RankingRecord, ranking_entry_id, ranking_entry_score};

pub fn entry_find_by_id(entries: &[Option<RankingRecord>], id: i32) -> i32 {
    for entry in entries {
        if ranking_entry_id(entry) == id {
            return ranking_entry_score(entry);
        }
    }

    0
}
