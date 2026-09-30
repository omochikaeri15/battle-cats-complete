//! Parsing of the Cat combo table.
//!
//! The engine declares one combo per row of `NyancomboData.csv` and writes that
//! combo's localized name on the matching line of `Nyancombo_<lang>.csv`, so a
//! row is addressed by its position in the file rather than by any column it
//! carries.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

/// Represents errors that can occur while parsing the Cat combo table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NyancomboDataError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for NyancomboDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid combo data."),
        }
    }
}

impl std::error::Error for NyancomboDataError {}

/// One member slot of a combo's lineup requirement.
///
/// The table reserves five slots per combo and fills the unused ones with the
/// absent sentinel, so a slot counts only when it names a real unit.
/// [`NyancomboData`] stores the slots as their own columns; this pairs them for
/// the accessors that present them together.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboSlot {
    /// The identifier of the unit the slot requires.
    pub unit_id: i32,
    /// The evolutionary form that unit must be in, counted from zero at the normal form.
    pub form: i32,
}

impl ComboSlot {
    /// Reports whether the slot names a real member.
    ///
    /// # Returns
    /// A `bool` that is true when the slot holds a unit identifier and a form,
    /// and false for the negative pair an unused slot carries.
    pub fn is_occupied(&self) -> bool {
        self.unit_id >= 0 && self.form >= 0
    }
}

/// The band a combo grants its effect at.
///
/// The engine writes the name of each band on the matching line of
/// `Nyancombo2_<lang>.csv`, and the magnitude the band awards is the column of
/// `NyancomboParam.tsv` at the same position, on the row the combo's effect type
/// selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComboStrength {
    /// The smallest band, written 小.
    Small,
    /// The middle band, written 中.
    Medium,
    /// The largest ordinary band, written 大.
    Large,
    /// The band above the ordinary three, written 究極.
    Ultimate,
    /// The band that lowers the effect instead of raising it, written ダウン.
    Down,
    /// The band that confers the effect outright rather than scaling it, written 付与.
    Grant,
    /// A band this parser does not recognize, carrying its raw value.
    Unknown(i32),
}

impl ComboStrength {
    /// The position this band occupies in the engine's band table.
    ///
    /// # Returns
    /// An `i32` holding the line of `Nyancombo2_<lang>.csv` that names the band,
    /// which is also the cell of the effect's `NyancomboParam.tsv` row that
    /// holds its magnitude.
    pub const fn index(self) -> i32 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
            Self::Ultimate => 3,
            Self::Down => 4,
            Self::Grant => 5,
            Self::Unknown(value) => value,
        }
    }
}

impl From<i32> for ComboStrength {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Small,
            1 => Self::Medium,
            2 => Self::Large,
            3 => Self::Ultimate,
            4 => Self::Down,
            5 => Self::Grant,
            _ => Self::Unknown(value),
        }
    }
}

/// One combo: the lineup that triggers it and the effect it grants.
///
/// Every field is one column of the raw table, in the order the table declares
/// them. The five member slots are available as an assembled view through
/// [`NyancomboData::slots`], and the effect band through
/// [`NyancomboData::strength`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NyancomboData {
    /// The combo's identifier, which restarts at zero within each series.
    pub combo_id: i32,
    /// The series the combo belongs to, or negative one on a row the engine has superseded.
    ///
    /// The series partitions the table, and [`NyancomboData::combo_id`] is
    /// unique only within one of its partitions. Prefer
    /// [`NyancomboData::key`] over reading either column alone.
    pub series: i32,
    /// The unit restriction group scoping the combo, or negative one when it applies to the whole roster.
    ///
    /// The identifier addresses a row of `Charagroup.csv`, whose own text column
    /// names the line the game prints to qualify the effect.
    pub charagroup_id: i32,
    /// The identifier of the unit in the first member slot.
    pub slot_1_unit_id: i32,
    /// The form that unit must be in for the first member slot.
    pub slot_1_form: i32,
    /// The identifier of the unit in the second member slot.
    pub slot_2_unit_id: i32,
    /// The form that unit must be in for the second member slot.
    pub slot_2_form: i32,
    /// The identifier of the unit in the third member slot.
    pub slot_3_unit_id: i32,
    /// The form that unit must be in for the third member slot.
    pub slot_3_form: i32,
    /// The identifier of the unit in the fourth member slot.
    pub slot_4_unit_id: i32,
    /// The form that unit must be in for the fourth member slot.
    pub slot_4_form: i32,
    /// The identifier of the unit in the fifth member slot.
    pub slot_5_unit_id: i32,
    /// The form that unit must be in for the fifth member slot.
    pub slot_5_form: i32,
    /// The effect the combo grants, as a row of `NyancomboParam.tsv` and a line of `Nyancombo1_<lang>.csv`.
    ///
    /// The same identifier appears in a map's `InvalidNyancomboID` list, which
    /// bars every combo granting that effect rather than one named combo.
    pub effect_type: i32,
    /// The band the effect is granted at, as a column of `NyancomboParam.tsv` and a line of `Nyancombo2_<lang>.csv`.
    ///
    /// Prefer [`NyancomboData::strength`] over reading this index directly.
    pub effect_level: i32,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_15: i32,
    /// Any trailing columns beyond the known layout, kept for forward compatibility.
    ///
    /// A column that does not read as an integer is held as `None` rather than
    /// discarded, so an element's index is always its offset past the layout.
    pub rest: Vec<Option<i32>>,
}

impl Default for NyancomboData {
    /// Produces a row in which every column is absent.
    ///
    /// Each field holds the value the matching column of
    /// [`NyancomboData::COLUMNS`] falls back to, so a row built this way names
    /// no combo, no members, and no effect.
    fn default() -> Self {
        Self {
            combo_id: -1,
            series: -1,
            charagroup_id: -1,
            slot_1_unit_id: -1,
            slot_1_form: -1,
            slot_2_unit_id: -1,
            slot_2_form: -1,
            slot_3_unit_id: -1,
            slot_3_form: -1,
            slot_4_unit_id: -1,
            slot_4_form: -1,
            slot_5_unit_id: -1,
            slot_5_form: -1,
            effect_type: -1,
            effect_level: -1,
            unknown_15: -1,
            rest: Vec::new(),
        }
    }
}

impl NyancomboData {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `NyancomboData.csv` row
    /// from the parser's own table instead of restating it. Every column falls
    /// back to the negative one the raw table uses to mean absent, and columns
    /// past the table are kept in [`NyancomboData::rest`].
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent -1;
        combo_id       : 0;
        series         : 1;
        charagroup_id  : 2;
        slot_1_unit_id : 3;
        slot_1_form    : 4;
        slot_2_unit_id : 5;
        slot_2_form    : 6;
        slot_3_unit_id : 7;
        slot_3_form    : 8;
        slot_4_unit_id : 9;
        slot_4_form    : 10;
        slot_5_unit_id : 11;
        slot_5_form    : 12;
        effect_type    : 13;
        effect_level   : 14;
        unknown_15     : 15;
    };

    /// The pair that addresses this combo within the table.
    ///
    /// # Returns
    /// A tuple whose first element is the series and whose second is the
    /// identifier the combo carries within it.
    pub fn key(&self) -> (i32, i32) {
        (self.series, self.combo_id)
    }

    /// Reports whether the engine still awards this combo.
    ///
    /// The table opens with a block of superseded rows, each carrying a negative
    /// series, which the file retains so that the localized name tables keep
    /// their line alignment. A superseded row usually restates a combo the live
    /// block declares again, at times with a different effect.
    ///
    /// # Returns
    /// A `bool` that is true when the row belongs to a live series.
    pub fn is_active(&self) -> bool {
        self.series > 0
    }

    /// Collects the five member slots in table order.
    ///
    /// # Returns
    /// An array holding every slot the table reserves, unused ones included.
    pub fn slots(&self) -> [ComboSlot; 5] {
        [
            ComboSlot { unit_id: self.slot_1_unit_id, form: self.slot_1_form },
            ComboSlot { unit_id: self.slot_2_unit_id, form: self.slot_2_form },
            ComboSlot { unit_id: self.slot_3_unit_id, form: self.slot_3_form },
            ComboSlot { unit_id: self.slot_4_unit_id, form: self.slot_4_form },
            ComboSlot { unit_id: self.slot_5_unit_id, form: self.slot_5_form },
        ]
    }

    /// Yields the combo's occupied member slots in table order.
    ///
    /// # Returns
    /// An iterator over the slots of [`NyancomboData::slots`] that name a real
    /// member.
    pub fn members(&self) -> impl Iterator<Item = ComboSlot> {
        self.slots().into_iter().filter(ComboSlot::is_occupied)
    }

    /// Decodes the band the combo grants its effect at.
    ///
    /// # Returns
    /// A `ComboStrength` naming the band, which carries the raw index when the
    /// column holds one the engine's band table does not list.
    pub fn strength(&self) -> ComboStrength {
        ComboStrength::from(self.effect_level)
    }

    fn from_csv_line(csv_line: &str, delimiter: char) -> Self {
        let parts: Vec<&str> = csv_line.split(delimiter).map(str::trim).collect();

        let mut row = Self::default();
        let past_table = columns::apply(&parts, Self::COLUMNS, &mut row);

        row.rest = parts
            .iter()
            .skip(past_table)
            .map(|part| part.parse::<i32>().ok())
            .collect();

        row
    }

    /// Parses the combo table into one row per line of the file.
    ///
    /// A row's position in the returned vector is its line number in the file,
    /// which is the line its localized name occupies in `Nyancombo_<lang>.csv`.
    /// A blank line therefore yields a row whose every column is absent rather
    /// than being dropped, so the two files stay aligned.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `NyancomboData.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows in file order on success, or a
    /// `NyancomboDataError` if the file carried no rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, NyancomboDataError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<NyancomboData>, NyancomboDataError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut rows = Vec::new();
    let mut has_content = false;

    for line in file_content.lines() {
        has_content |= !line.trim().is_empty();
        rows.push(NyancomboData::from_csv_line(line, delimiter));
    }

    if !has_content {
        return Err(NyancomboDataError::EmptyFile);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLUMN_COUNT: usize = 16;

    /// Line 126 of a real `NyancomboData.csv`, the first live row and one of the
    /// few that fills every member slot.
    const REAL_ROW: &str = "0,1,-1,0,0,1,0,2,0,3,0,4,0,4,0,-1";

    /// Line 0 of the same file, the superseded restatement of `REAL_ROW`.
    const SUPERSEDED_ROW: &str = "0,-1,-1,0,0,1,0,2,0,3,0,4,0,5,0,-1";

    /// Line 381 of the same file, a restricted combo granting its effect outright.
    const RESTRICTED_ROW: &str = "106,1,10,857,0,858,0,-1,-1,-1,-1,-1,-1,28,5,-1";

    /// Every field of [`NyancomboData`] in the order its column appears in the row.
    const FIELD_ORDER: [&str; COLUMN_COUNT] = [
        "combo_id",
        "series",
        "charagroup_id",
        "slot_1_unit_id",
        "slot_1_form",
        "slot_2_unit_id",
        "slot_2_form",
        "slot_3_unit_id",
        "slot_3_form",
        "slot_4_unit_id",
        "slot_4_form",
        "slot_5_unit_id",
        "slot_5_form",
        "effect_type",
        "effect_level",
        "unknown_15",
    ];

    fn parse_one(line: &str) -> NyancomboData {
        NyancomboData::parse(line, None).unwrap().remove(0)
    }

    fn fields_set_by(index: usize) -> Vec<String> {
        let mut columns = vec!["0"; COLUMN_COUNT];
        columns[index] = "7";

        let probed = serde_json::to_value(parse_one(&columns.join(","))).unwrap();
        let baseline = serde_json::to_value(parse_one(&vec!["0"; COLUMN_COUNT].join(","))).unwrap();

        let (Some(probed), Some(baseline)) = (probed.as_object(), baseline.as_object()) else {
            return Vec::new();
        };

        probed
            .iter()
            .filter(|(key, value)| baseline.get(*key) != Some(*value))
            .map(|(key, _)| key.clone())
            .collect()
    }

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        let mut reached = Vec::new();

        for index in 0..COLUMN_COUNT {
            let touched = fields_set_by(index);
            assert_eq!(touched.len(), 1, "column {index} set {touched:?}");
            reached.push(touched.into_iter().next().unwrap_or_default());
        }

        assert_eq!(reached, FIELD_ORDER);

        reached.sort();
        reached.dedup();
        assert_eq!(reached.len(), COLUMN_COUNT);
    }

    #[test]
    fn a_real_row_lands_column_for_column() {
        let row = parse_one(REAL_ROW);

        assert_eq!(row.combo_id, 0, "combo_id");
        assert_eq!(row.series, 1, "series");
        assert_eq!(row.charagroup_id, -1, "charagroup_id");
        assert_eq!(row.slot_1_unit_id, 0, "slot_1_unit_id");
        assert_eq!(row.slot_1_form, 0, "slot_1_form");
        assert_eq!(row.slot_2_unit_id, 1, "slot_2_unit_id");
        assert_eq!(row.slot_2_form, 0, "slot_2_form");
        assert_eq!(row.slot_3_unit_id, 2, "slot_3_unit_id");
        assert_eq!(row.slot_3_form, 0, "slot_3_form");
        assert_eq!(row.slot_4_unit_id, 3, "slot_4_unit_id");
        assert_eq!(row.slot_4_form, 0, "slot_4_form");
        assert_eq!(row.slot_5_unit_id, 4, "slot_5_unit_id");
        assert_eq!(row.slot_5_form, 0, "slot_5_form");
        assert_eq!(row.effect_type, 4, "effect_type");
        assert_eq!(row.effect_level, 0, "effect_level");
        assert_eq!(row.unknown_15, -1, "unknown_15");
        assert!(row.rest.is_empty());
    }

    #[test]
    fn the_assembled_views_agree_with_the_raw_columns() {
        let row = parse_one(REAL_ROW);

        assert_eq!(row.key(), (1, 0));
        assert!(row.is_active());
        assert_eq!(row.strength(), ComboStrength::Small);
        assert_eq!(row.members().count(), 5);
        assert_eq!(
            row.slots().map(|slot| slot.unit_id),
            [
                row.slot_1_unit_id,
                row.slot_2_unit_id,
                row.slot_3_unit_id,
                row.slot_4_unit_id,
                row.slot_5_unit_id,
            ],
        );

        let restricted = parse_one(RESTRICTED_ROW);

        assert_eq!(restricted.key(), (1, 106));
        assert_eq!(restricted.charagroup_id, 10);
        assert_eq!(restricted.effect_type, 28);
        assert_eq!(restricted.strength(), ComboStrength::Grant);
        assert_eq!(
            restricted.members().collect::<Vec<_>>(),
            [
                ComboSlot { unit_id: 857, form: 0 },
                ComboSlot { unit_id: 858, form: 0 },
            ],
        );
        assert!(restricted.slots()[2..].iter().all(|slot| !slot.is_occupied()));
    }

    #[test]
    fn a_superseded_row_restates_a_live_one_it_does_not_replace() {
        let superseded = parse_one(SUPERSEDED_ROW);
        let live = parse_one(REAL_ROW);

        assert!(!superseded.is_active());
        assert_eq!(superseded.key(), (-1, 0));
        assert_eq!(superseded.combo_id, live.combo_id);
        assert_eq!(superseded.slots(), live.slots());
        assert_ne!(superseded.effect_type, live.effect_type);
    }

    #[test]
    fn an_unrecognized_band_keeps_its_raw_index() {
        let row = parse_one("0,1,-1,0,0,-1,-1,-1,-1,-1,-1,-1,-1,4,9,-1");

        assert_eq!(row.strength(), ComboStrength::Unknown(9));
        assert_eq!(row.members().collect::<Vec<_>>(), [ComboSlot { unit_id: 0, form: 0 }]);
    }

    #[test]
    fn a_short_row_leaves_its_missing_columns_absent() {
        let row = parse_one("7,6,-1,13,1");

        assert_eq!(row.key(), (6, 7));
        assert_eq!(row.slots()[0], ComboSlot { unit_id: 13, form: 1 });
        assert_eq!(row.members().count(), 1);
        assert_eq!(row.effect_type, -1);
        assert_eq!(row.strength(), ComboStrength::Unknown(-1));
        assert!(row.rest.is_empty());
    }

    #[test]
    fn a_trailing_column_is_kept_rather_than_dropped() {
        let row = parse_one("0,1,-1,0,0,-1,-1,-1,-1,-1,-1,-1,-1,4,0,-1,12,x");

        assert_eq!(row.rest, [Some(12), None]);
    }

    #[test]
    fn a_blank_line_holds_its_place_in_the_file() {
        let rows = NyancomboData::parse(format!("{REAL_ROW}\n\n{RESTRICTED_ROW}"), None).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1], NyancomboData::default());
        assert_eq!(rows[2].combo_id, 106);
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(NyancomboData::parse("", None), Err(NyancomboDataError::EmptyFile));
        assert_eq!(NyancomboData::parse("\n\n", None), Err(NyancomboDataError::EmptyFile));
    }
}
