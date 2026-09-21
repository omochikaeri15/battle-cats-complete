use std::collections::HashMap;

const COMMENT: &str = "//";
const STORY_TAIL: usize = 2;
const NAME_COLUMN: usize = 1;
const STORY_LAST_NAMED: u32 = 47;
const STORY_PREFIXES: [&str; 4] = ["EC", "W", "Space", "Z"];

pub fn named_by_stage(prefix: &str) -> bool {
    STORY_PREFIXES.contains(&prefix)
}

pub fn stage_name_address(prefix: &str, map_id: u32, stage_id: u32) -> (u32, usize) {
    if !named_by_stage(prefix) {
        return (map_id, stage_id as usize);
    }

    let grouped = prefix == "EC" || (prefix == "Z" && map_id <= 2);

    match grouped && matches!(stage_id, 48..=50) {
        true => (STORY_LAST_NAMED, 0),
        false => (stage_id, 0),
    }
}

pub fn map_name_rows(body: &str, delimiter: char) -> HashMap<u32, usize> {
    let mut rows = HashMap::new();

    for (index, line) in body.lines().enumerate() {
        let Some(cells) = columns(line, delimiter) else { continue; };

        if cells.len() <= NAME_COLUMN {
            continue;
        }

        let Ok(map_id) = cells[0].trim().parse::<u32>() else { continue; };

        rows.insert(map_id, index);
    }

    rows
}

pub fn stage_name_rows(body: &str, delimiter: char) -> HashMap<u32, usize> {
    let mut scanned: Vec<(usize, String)> = Vec::new();

    for (index, line) in body.lines().enumerate() {
        let Some(cells) = columns(line, delimiter) else { continue; };

        scanned.push((index, cells.first().map(|cell| cell.trim().to_owned()).unwrap_or_default()));
    }

    let Some(dummy) = scanned.iter().position(|(_, leading)| placeholder(leading)) else {
        return scanned.into_iter().map(|(index, _)| (index as u32, index)).collect();
    };

    let valid: Vec<usize> = scanned.into_iter().take(dummy).map(|(index, _)| index).collect();

    if valid.len() < STORY_TAIL {
        return valid.into_iter().enumerate().map(|(key, index)| (key as u32, index)).collect();
    }

    let (backwards, forwards) = valid.split_at(valid.len() - STORY_TAIL);

    backwards
        .iter()
        .rev()
        .chain(forwards)
        .enumerate()
        .map(|(key, index)| (key as u32, *index))
        .collect()
}

fn columns(line: &str, delimiter: char) -> Option<Vec<&str>> {
    let clean = line.split_once(COMMENT).map_or(line, |(before, _)| before);
    let trimmed = clean.trim();

    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.split(delimiter).collect())
}

fn placeholder(cell: &str) -> bool {
    let clean = cell.trim();

    clean.eq_ignore_ascii_case("dammy")
        || clean.eq_ignore_ascii_case("<tbd>")
        || clean == "預備"
        || clean == "예비"
        || clean == "予備"
}

#[cfg(test)]
mod tests {
    use super::*;

    // Empire of Cats reuses stage 47's name for 48, 49 and 50, and only there. Getting
    // this wrong renames a neighbouring stage rather than failing, so it is pinned.
    #[test]
    fn the_story_chapters_read_one_name_per_line() {
        assert_eq!(stage_name_address("EC", 0, 12), (12, 0));
        assert_eq!(stage_name_address("EC", 0, 49), (47, 0));
        assert_eq!(stage_name_address("Z", 1, 49), (47, 0));

        // Zombie maps above 2 borrow no story chapter, so 48..=50 stay themselves.
        assert_eq!(stage_name_address("Z", 5, 49), (49, 0));

        // Every other chapter holds a whole map on one line and reads the stage's cell.
        assert_eq!(stage_name_address("V", 3, 7), (3, 7));
    }
}
