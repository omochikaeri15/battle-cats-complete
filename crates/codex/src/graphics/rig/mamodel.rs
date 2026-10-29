use serde::Serialize;

use crate::common::columns::{self, Column};
use crate::common::stream::{declared_rows, Reader};

use super::RigError;

const GLOW_VERSION: i32 = 2;

pub(crate) const ALIGNED_VERSION: i32 = 3;

const UNVERSIONED_SCALE: i32 = 100;

const UNVERSIONED_ANGLE: i32 = 360;

const UNVERSIONED_OPACITY: i32 = 255;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ModelPart {
    pub parent: i32,
    pub id: i32,
    pub sprite: i32,
    pub z: i32,
    pub x: i32,
    pub y: i32,
    pub pivot_x: i32,
    pub pivot_y: i32,
    pub scale_x: i32,
    pub scale_y: i32,
    pub angle: i32,
    pub opacity: i32,
    pub glow: i32,
    pub name: String,
}

impl ModelPart {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        parent   :  0;
        id       :  1;
        sprite   :  2;
        z        :  3;
        x        :  4;
        y        :  5;
        pivot_x  :  6;
        pivot_y  :  7;
        scale_x  :  8;
        scale_y  :  9;
        angle    : 10;
        opacity  : 11;
        glow     : 12;
    };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Alignment {
    pub part: i32,
    pub unknown_1: i32,
    pub x: i32,
    pub y: i32,
    pub unknown_4: i32,
    pub unknown_5: i32,
    pub name: String,
}

impl Alignment {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        part      : 0;
        unknown_1 : 1;
        x         : 2;
        y         : 3;
        unknown_4 : 4;
        unknown_5 : 5;
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub version: i32,
    pub parts: Vec<ModelPart>,
    pub scale_unit: i32,
    pub angle_unit: i32,
    pub opacity_unit: i32,
    pub unknown_3: Option<i32>,
    pub alignment: Vec<Alignment>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            version: 0,
            parts: Vec::new(),
            scale_unit: 1000,
            angle_unit: 3600,
            opacity_unit: 1000,
            unknown_3: None,
            alignment: Vec::new(),
        }
    }
}

impl Model {
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, RigError> {
        Self::parse_inner(bytes.as_ref())
    }

    fn parse_inner(bytes: &[u8]) -> Result<Self, RigError> {
        let content = String::from_utf8_lossy(bytes);
        let mut reader = Reader::new(&content);

        if reader.line().is_none() { return Err(RigError::EmptyFile); }

        reader.row();
        let version = reader.value(0);

        reader.row();
        let count = declared_rows(reader.value(0)).ok_or(RigError::CountTooLarge)?;

        if count == 0 { return Err(RigError::NoPartHeader); }

        let mut parts = Vec::with_capacity(count);

        for _ in 0..count {
            reader.row();

            let mut part = ModelPart::default();
            let trailing = columns::apply(&reader.scalars(), ModelPart::COLUMNS, &mut part);

            if version < GLOW_VERSION { part.glow = 0; }
            part.name = reader.cells().get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            parts.push(part);
        }

        let mut model = Model { version, parts, ..Model::default() };

        if version <= 0 {
            model.scale_unit = UNVERSIONED_SCALE;
            model.angle_unit = UNVERSIONED_ANGLE;
            model.opacity_unit = UNVERSIONED_OPACITY;

            return Ok(model);
        }

        reader.row();
        model.scale_unit = reader.value(0);
        model.angle_unit = reader.value(1);
        model.opacity_unit = reader.value(2);
        model.unknown_3 = (reader.cells().len() > 3).then(|| reader.value(3));

        if version < ALIGNED_VERSION { return Ok(model); }

        reader.row();
        let count = declared_rows(reader.value(0)).ok_or(RigError::CountTooLarge)?;

        model.alignment = Vec::with_capacity(count);

        for _ in 0..count {
            reader.row();

            let mut alignment = Alignment::default();
            let trailing = columns::apply(&reader.scalars(), Alignment::COLUMNS, &mut alignment);
            alignment.name = reader.cells().get(trailing).map(|text| text.trim().to_string()).unwrap_or_default();

            model.alignment.push(alignment);
        }

        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PART: &str = "0,1,2,3,4,5,6,7,8,9,10,11,12,body";

    fn file(version: i32, tail: &[&str]) -> String {
        let mut lines = vec!["[modelanim:model]".to_string(), version.to_string(), "1".to_string(), PART.to_string()];
        lines.extend(tail.iter().map(|line| (*line).to_string()));

        lines.join("\n")
    }

    #[test]
    fn a_version_below_one_takes_the_engines_own_divisors_and_reads_no_further() {
        let model = Model::parse(file(0, &["7,8,9", "1", "0,0,10,20,0,0"])).expect("a model parses");

        assert_eq!(
            (model.scale_unit, model.angle_unit, model.opacity_unit, model.unknown_3),
            (UNVERSIONED_SCALE, UNVERSIONED_ANGLE, UNVERSIONED_OPACITY, None),
        );
        assert_eq!(model.alignment, Vec::new());
    }

    #[test]
    fn a_version_below_two_carries_no_blending_column() {
        assert_eq!(Model::parse(file(1, &["7,8,9"])).map(|model| model.parts[0].glow), Ok(0));
        assert_eq!(Model::parse(file(2, &["7,8,9"])).map(|model| model.parts[0].glow), Ok(12));
    }

    #[test]
    fn a_version_below_three_reads_the_units_row_and_stops() {
        let model = Model::parse(file(2, &["7,8,9,10", "1", "0,0,10,20,0,0"])).expect("a model parses");

        assert_eq!((model.scale_unit, model.angle_unit, model.opacity_unit), (7, 8, 9));
        assert_eq!(model.unknown_3, Some(10));
        assert_eq!(model.alignment, Vec::new());
    }

    #[test]
    fn a_blank_line_is_a_row_like_any_other() {
        let model = Model::parse(file(3, &["", "7,8,9", "0"])).expect("a model parses");

        assert_eq!((model.scale_unit, model.angle_unit, model.opacity_unit), (0, 0, 0));
    }

    #[test]
    fn a_file_that_runs_out_repeats_the_last_row_it_read() {
        let truncated = "[modelanim:model]\n3\n3\n0,1,2,3,4,5,6,7,8,9,10,11,12,body";
        let model = Model::parse(truncated).expect("a model parses");

        assert_eq!(model.parts.len(), 3);
        assert_eq!(model.parts[1], model.parts[0]);
        assert_eq!(model.parts[2], model.parts[0]);
    }

    #[test]
    fn a_count_the_engine_aborts_on_is_refused() {
        assert_eq!(Model::parse("[modelanim:model]\n3\n-1"), Err(RigError::CountTooLarge));
        assert_eq!(Model::parse(file(3, &["7,8,9", "-4"])), Err(RigError::CountTooLarge));
        assert_eq!(Model::parse(""), Err(RigError::EmptyFile));
        assert_eq!(Model::parse("[modelanim:model]\n3\n0"), Err(RigError::NoPartHeader));
    }

    #[test]
    fn part_columns_map_one_field_each() {
        columns::assert_one_field_per_column(ModelPart::COLUMNS);
    }

    #[test]
    fn alignment_columns_map_one_field_each() {
        columns::assert_one_field_per_column(Alignment::COLUMNS);
    }
}
