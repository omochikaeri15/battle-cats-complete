use super::{ranking_entry_id, ranking_record_rank, RankingRecord};

pub fn ranking_rank_by_id(entries: &[Option<RankingRecord>], id: i32) -> i32 {
    let mut record = None;

    for entry in entries {
        if ranking_entry_id(entry) == id {
            record = entry.as_ref();

            break;
        }
    }

    record.map_or(0, ranking_record_rank)
}
