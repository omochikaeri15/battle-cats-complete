use super::{ranking_record_name, RankingRecord};

pub fn ranking_entry_name(entry: &Option<RankingRecord>) -> Vec<u8> {
    entry.as_ref().map_or_else(Vec::new, ranking_record_name)
}
