use std::collections::BTreeSet;

use crate::Vfs;

pub const FILE: &str = "StagePointRulesMap.json";

pub fn score_stage_maps(vfs: &Vfs) -> BTreeSet<i32> {
    vfs.load(FILE).map_or_else(BTreeSet::new, |bytes| parse(&bytes))
}

pub fn parse(bytes: &[u8]) -> BTreeSet<i32> {
    let Ok(document) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return BTreeSet::new();
    };
    let Some(maps) = document.get("MapID").and_then(serde_json::Value::as_object) else {
        return BTreeSet::new();
    };

    maps.keys().filter_map(|key| key.trim().parse().ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_ids_come_back_as_numbers() {
        let listed = parse(br#"{"MapID":{"39000":{"StageIndex":{}},"33000":{}}}"#);

        assert_eq!(listed, BTreeSet::from([33000, 39000]));
    }

    #[test]
    fn a_file_that_is_missing_or_shaped_wrong_restricts_nothing() {
        assert!(parse(b"").is_empty());
        assert!(parse(b"{}").is_empty());
        // A modder rewriting the file must not be able to make every stage
        // look restricted by accident.
        assert!(parse(br#"{"MapID":[]}"#).is_empty());
        assert!(parse(br#"{"MapID":{"nonsense":{}}}"#).is_empty());
    }
}
