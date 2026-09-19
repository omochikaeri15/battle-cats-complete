use super::{RankingRecord, ranking_entry_id, ranking_entry_name};

pub fn ranking_name_by_id(entries: &[Option<RankingRecord>], id: i32) -> Vec<u8> {
    for entry in entries {
        if ranking_entry_id(entry) == id {
            return ranking_entry_name(entry);
        }
    }

    Vec::new()
}
