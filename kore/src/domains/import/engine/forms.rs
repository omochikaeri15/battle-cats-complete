pub(crate) const FILE: &str = "unitbuy.csv";

const CELLS: usize = 63;
const TRUE_FORM_ID: usize = 23;
const ULTRA_FORM_ID: usize = 24;
const FORM_BLOCK: std::ops::RangeInclusive<usize> = 23..=48;
const IDENTITY: [usize; 12] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14];

fn cells(line: &str) -> Vec<&str> {
    line.split(',').collect()
}

fn number(cell: &str) -> Option<i64> {
    cell.trim().parse().ok()
}

fn declares_a_form(row: &[&str]) -> bool {
    [TRUE_FORM_ID, ULTRA_FORM_ID]
        .iter()
        .filter_map(|column| row.get(*column).copied().and_then(number))
        .any(|id| id != 0)
}

fn same_unit(winner: &[&str], donor: &[&str]) -> bool {
    IDENTITY.iter().all(|column| {
        let held = winner.get(*column).copied().and_then(number);
        let offered = donor.get(*column).copied().and_then(number);

        held.is_some() && held == offered
    })
}

pub(crate) fn adopt_declared_forms(winner: &[u8], donors: &[Vec<u8>]) -> Option<(Vec<u8>, Vec<usize>)> {
    let held = String::from_utf8(winner.to_vec()).ok()?;
    let offered: Vec<Vec<String>> = donors
        .iter()
        .filter_map(|donor| String::from_utf8(donor.to_vec()).ok())
        .map(|text| text.lines().map(str::to_owned).collect())
        .collect();
    let mut rows: Vec<String> = held.lines().map(str::to_owned).collect();
    let mut filled = Vec::new();

    for (unit, row) in rows.iter_mut().enumerate() {
        let mut winning = cells(row);

        if winning.len() != CELLS || declares_a_form(&winning) {
            continue;
        }

        let Some(donor) = offered
            .iter()
            .filter_map(|lines| lines.get(unit))
            .map(|line| cells(line))
            .find(|donor| donor.len() == CELLS && declares_a_form(donor) && same_unit(&winning, donor))
        else {
            continue;
        };

        for column in FORM_BLOCK {
            if let Some(cell) = winning.get_mut(column)
                && let Some(value) = donor.get(column)
            {
                *cell = value;
            }
        }

        *row = winning.join(",");
        filled.push(unit);
    }

    if filled.is_empty() {
        return None;
    }

    let mut merged = rows.join("\n");

    if held.ends_with('\n') {
        merged.push('\n');
    }

    Some((merged.into_bytes(), filled))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(unit: i64, true_form: i64) -> String {
        let mut cells = vec!["0".to_owned(); CELLS];
        // The identity columns are the unit's own economy, which every region agrees on.
        for (at, column) in IDENTITY.iter().enumerate() {
            cells[*column] = (unit * 100 + at as i64).to_string();
        }
        cells[TRUE_FORM_ID] = true_form.to_string();
        cells[25] = if true_form == 0 { "-1".to_owned() } else { "30".to_owned() };
        cells.join(",")
    }

    fn table(forms: &[i64]) -> Vec<u8> {
        offset_table(forms, 0)
    }

    fn offset_table(forms: &[i64], offset: i64) -> Vec<u8> {
        forms
            .iter()
            .enumerate()
            .map(|(unit, form)| row(unit as i64 + offset, *form))
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes()
    }

    #[test]
    fn a_form_the_winner_lacks_is_adopted() {
        let winner = table(&[0, 0, 0]);
        let donor = table(&[0, 10078, 0]);
        let (merged, filled) = adopt_declared_forms(&winner, &[donor]).expect("a row to fill");

        assert_eq!(filled, vec![1]);

        let rows: Vec<&str> = std::str::from_utf8(&merged).unwrap_or_default().lines().collect();
        assert_eq!(cells(rows[1])[TRUE_FORM_ID], "10078");
        // The companion columns travel with the id, or the form is unreachable.
        assert_eq!(cells(rows[1])[25], "30");
    }

    #[test]
    fn a_winner_that_already_has_the_form_is_left_alone() {
        let winner = table(&[0, 15472, 0]);
        let donor = table(&[0, 15472, 0]);

        assert!(adopt_declared_forms(&winner, &[donor]).is_none());
    }

    #[test]
    fn nothing_happens_without_a_donor() {
        assert!(adopt_declared_forms(&table(&[0, 0]), &[]).is_none());
    }

    #[test]
    fn a_row_describing_a_different_unit_is_refused() {
        let winner = table(&[0, 0, 0]);
        // Every donor row describes the next unit along, which stands in for a
        // build where the table gained a unit. Splicing here would put one
        // unit's form onto another.
        let donor = offset_table(&[10078, 10079, 10080], 1);

        assert!(adopt_declared_forms(&winner, &[donor]).is_none());
    }

    #[test]
    fn the_trailing_newline_survives() {
        let mut winner = table(&[0]);
        winner.push(b'\n');
        let donor = table(&[10078]);
        let (merged, _) = adopt_declared_forms(&winner, &[donor]).expect("a row to fill");

        assert!(merged.ends_with(b"\n"));
    }
}
