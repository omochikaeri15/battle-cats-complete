use std::collections::HashMap;
use std::fmt;

use crate::common::columns;
use crate::common::columns::Column;
use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitBuyError {
    EmptyFile,
}

impl fmt::Display for UnitBuyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                f,
                "The provided file bytes contained no valid unit buy data."
            ),
        }
    }
}

impl std::error::Error for UnitBuyError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvolveMaterial {
    pub item_id: i32,
    pub quantity: i32,
}

impl EvolveMaterial {
    pub fn is_occupied(&self) -> bool {
        self.item_id != 0 && self.quantity > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitBuy {
    pub stage_unlock_requirement: i32,
    pub purchase_cost: i32,
    pub upgrade_cost_1: i32,
    pub upgrade_cost_2: i32,
    pub upgrade_cost_3: i32,
    pub upgrade_cost_4: i32,
    pub upgrade_cost_5: i32,
    pub upgrade_cost_6: i32,
    pub upgrade_cost_7: i32,
    pub upgrade_cost_8: i32,
    pub upgrade_cost_9: i32,
    pub upgrade_cost_10: i32,
    pub currency_type: i32,
    pub rarity: i32,
    pub guide_order: i32,
    pub chapter_unlock_requirement: i32,
    pub sell_xp_yield: i32,
    pub fanfare_tier: i32,
    pub level_cap_ch2: i32,
    pub base_max_plus_level: i32,
    pub true_form_obtain_level: i32,
    pub evolved_form_unlock_level: i32,
    pub level_cap_ch1: i32,
    pub true_form_id: i32,
    pub ultra_form_id: i32,
    pub true_form_unlock_level: i32,
    pub ultra_form_unlock_level: i32,
    pub true_form_xp_cost: i32,
    pub true_form_material_1_id: i32,
    pub true_form_material_1_quantity: i32,
    pub true_form_material_2_id: i32,
    pub true_form_material_2_quantity: i32,
    pub true_form_material_3_id: i32,
    pub true_form_material_3_quantity: i32,
    pub true_form_material_4_id: i32,
    pub true_form_material_4_quantity: i32,
    pub true_form_material_5_id: i32,
    pub true_form_material_5_quantity: i32,
    pub ultra_form_xp_cost: i32,
    pub ultra_form_material_1_id: i32,
    pub ultra_form_material_1_quantity: i32,
    pub ultra_form_material_2_id: i32,
    pub ultra_form_material_2_quantity: i32,
    pub ultra_form_material_3_id: i32,
    pub ultra_form_material_3_quantity: i32,
    pub ultra_form_material_4_id: i32,
    pub ultra_form_material_4_quantity: i32,
    pub ultra_form_material_5_id: i32,
    pub ultra_form_material_5_quantity: i32,
    pub level_cap_standard: i32,
    pub level_cap_catseye: i32,
    pub level_cap_plus: i32,
    pub normal_evolution_y_offset: i32,
    pub evolved_evolution_y_offset: i32,
    pub true_evolution_y_offset: i32,
    pub ultra_evolution_y_offset: i32,
    pub item_powerup_group: i32,
    pub version_added: i32,
    pub sell_np_yield: i32,
    pub true_form_event_id: i32,
    pub plus_cap_grows: i32,
    pub egg_id_normal: i32,
    pub egg_id_evolved: i32,
    pub rest: Vec<Option<i32>>,
}

impl Default for UnitBuy {
    fn default() -> Self {
        Self {
            stage_unlock_requirement: 0,
            purchase_cost: 0,
            upgrade_cost_1: 0,
            upgrade_cost_2: 0,
            upgrade_cost_3: 0,
            upgrade_cost_4: 0,
            upgrade_cost_5: 0,
            upgrade_cost_6: 0,
            upgrade_cost_7: 0,
            upgrade_cost_8: 0,
            upgrade_cost_9: 0,
            upgrade_cost_10: 0,
            currency_type: 0,
            rarity: 0,
            guide_order: -1,
            chapter_unlock_requirement: 0,
            sell_xp_yield: 0,
            fanfare_tier: 0,
            level_cap_ch2: 0,
            base_max_plus_level: 0,
            true_form_obtain_level: -1,
            evolved_form_unlock_level: 0,
            level_cap_ch1: 0,
            true_form_id: 0,
            ultra_form_id: 0,
            true_form_unlock_level: -1,
            ultra_form_unlock_level: -1,
            true_form_xp_cost: 0,
            true_form_material_1_id: 0,
            true_form_material_1_quantity: 0,
            true_form_material_2_id: 0,
            true_form_material_2_quantity: 0,
            true_form_material_3_id: 0,
            true_form_material_3_quantity: 0,
            true_form_material_4_id: 0,
            true_form_material_4_quantity: 0,
            true_form_material_5_id: 0,
            true_form_material_5_quantity: 0,
            ultra_form_xp_cost: 0,
            ultra_form_material_1_id: 0,
            ultra_form_material_1_quantity: 0,
            ultra_form_material_2_id: 0,
            ultra_form_material_2_quantity: 0,
            ultra_form_material_3_id: 0,
            ultra_form_material_3_quantity: 0,
            ultra_form_material_4_id: 0,
            ultra_form_material_4_quantity: 0,
            ultra_form_material_5_id: 0,
            ultra_form_material_5_quantity: 0,
            level_cap_standard: -1,
            level_cap_catseye: 0,
            level_cap_plus: 0,
            normal_evolution_y_offset: 0,
            evolved_evolution_y_offset: 0,
            true_evolution_y_offset: 0,
            ultra_evolution_y_offset: 0,
            item_powerup_group: 0,
            version_added: -1,
            sell_np_yield: 0,
            true_form_event_id: 0,
            plus_cap_grows: 0,
            egg_id_normal: -1,
            egg_id_evolved: -1,
            rest: Vec::new(),
        }
    }
}

impl UnitBuy {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent 0;
        stage_unlock_requirement       : 0;
        purchase_cost                  : 1;
        upgrade_cost_1                 : 2;
        upgrade_cost_2                 : 3;
        upgrade_cost_3                 : 4;
        upgrade_cost_4                 : 5;
        upgrade_cost_5                 : 6;
        upgrade_cost_6                 : 7;
        upgrade_cost_7                 : 8;
        upgrade_cost_8                 : 9;
        upgrade_cost_9                 : 10;
        upgrade_cost_10                : 11;
        currency_type                  : 12;
        rarity                         : 13;
        guide_order                    : 14, Raw, -1;
        chapter_unlock_requirement     : 15;
        sell_xp_yield                  : 16;
        fanfare_tier                   : 17;
        level_cap_ch2                  : 18;
        base_max_plus_level            : 19;
        true_form_obtain_level                : 20, Raw, -1;
        evolved_form_unlock_level      : 21;
        level_cap_ch1                  : 22;
        true_form_id                   : 23;
        ultra_form_id                  : 24;
        true_form_unlock_level         : 25, Raw, -1;
        ultra_form_unlock_level        : 26, Raw, -1;
        true_form_xp_cost              : 27;
        true_form_material_1_id        : 28;
        true_form_material_1_quantity  : 29;
        true_form_material_2_id        : 30;
        true_form_material_2_quantity  : 31;
        true_form_material_3_id        : 32;
        true_form_material_3_quantity  : 33;
        true_form_material_4_id        : 34;
        true_form_material_4_quantity  : 35;
        true_form_material_5_id        : 36;
        true_form_material_5_quantity  : 37;
        ultra_form_xp_cost             : 38;
        ultra_form_material_1_id       : 39;
        ultra_form_material_1_quantity : 40;
        ultra_form_material_2_id       : 41;
        ultra_form_material_2_quantity : 42;
        ultra_form_material_3_id       : 43;
        ultra_form_material_3_quantity : 44;
        ultra_form_material_4_id       : 45;
        ultra_form_material_4_quantity : 46;
        ultra_form_material_5_id       : 47;
        ultra_form_material_5_quantity : 48;
        level_cap_standard             : 49, Raw, -1;
        level_cap_catseye              : 50;
        level_cap_plus                 : 51;
        normal_evolution_y_offset      : 52;
        evolved_evolution_y_offset     : 53;
        true_evolution_y_offset        : 54;
        ultra_evolution_y_offset       : 55;
        item_powerup_group             : 56;
        version_added                  : 57, Raw, -1;
        sell_np_yield                  : 58;
        true_form_event_id             : 59;
        plus_cap_grows                 : 60;
        egg_id_normal                  : 61, Raw, -1;
        egg_id_evolved                 : 62, Raw, -1;
    };

    pub fn egg_ids(&self) -> (Option<u16>, Option<u16>) {
        (
            u16::try_from(self.egg_id_normal).ok(),
            u16::try_from(self.egg_id_evolved).ok(),
        )
    }

    pub fn upgrade_costs(&self) -> [i32; 10] {
        [
            self.upgrade_cost_1,
            self.upgrade_cost_2,
            self.upgrade_cost_3,
            self.upgrade_cost_4,
            self.upgrade_cost_5,
            self.upgrade_cost_6,
            self.upgrade_cost_7,
            self.upgrade_cost_8,
            self.upgrade_cost_9,
            self.upgrade_cost_10,
        ]
    }

    pub fn true_form_material_slots(&self) -> [EvolveMaterial; 5] {
        [
            EvolveMaterial {
                item_id: self.true_form_material_1_id,
                quantity: self.true_form_material_1_quantity,
            },
            EvolveMaterial {
                item_id: self.true_form_material_2_id,
                quantity: self.true_form_material_2_quantity,
            },
            EvolveMaterial {
                item_id: self.true_form_material_3_id,
                quantity: self.true_form_material_3_quantity,
            },
            EvolveMaterial {
                item_id: self.true_form_material_4_id,
                quantity: self.true_form_material_4_quantity,
            },
            EvolveMaterial {
                item_id: self.true_form_material_5_id,
                quantity: self.true_form_material_5_quantity,
            },
        ]
    }

    pub fn ultra_form_material_slots(&self) -> [EvolveMaterial; 5] {
        [
            EvolveMaterial {
                item_id: self.ultra_form_material_1_id,
                quantity: self.ultra_form_material_1_quantity,
            },
            EvolveMaterial {
                item_id: self.ultra_form_material_2_id,
                quantity: self.ultra_form_material_2_quantity,
            },
            EvolveMaterial {
                item_id: self.ultra_form_material_3_id,
                quantity: self.ultra_form_material_3_quantity,
            },
            EvolveMaterial {
                item_id: self.ultra_form_material_4_id,
                quantity: self.ultra_form_material_4_quantity,
            },
            EvolveMaterial {
                item_id: self.ultra_form_material_5_id,
                quantity: self.ultra_form_material_5_quantity,
            },
        ]
    }

    pub fn true_form_materials(&self) -> impl Iterator<Item = EvolveMaterial> {
        self.true_form_material_slots()
            .into_iter()
            .filter(EvolveMaterial::is_occupied)
    }

    pub fn ultra_form_materials(&self) -> impl Iterator<Item = EvolveMaterial> {
        self.ultra_form_material_slots()
            .into_iter()
            .filter(EvolveMaterial::is_occupied)
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

    pub fn parse<B: AsRef<[u8]>>(
        bytes: B,
        separator: Option<Separator>,
    ) -> Result<HashMap<u32, Self>, UnitBuyError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(
    bytes: &[u8],
    separator: Option<Separator>,
) -> Result<HashMap<u32, UnitBuy>, UnitBuyError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut map = HashMap::new();

    for (line_index, line) in file_content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        map.insert(line_index as u32, UnitBuy::from_csv_line(line, delimiter));
    }

    if map.is_empty() {
        return Err(UnitBuyError::EmptyFile);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLUMN_COUNT: usize = 63;

    const REAL_ROW: &str = "0,0,50,100,3100,7100,12100,18100,25100,33100,42100,52100,0,1,1000150,0,999,3,20,0,-1,10,10,15017,0,30,-1,500000,31,3,32,3,34,3,43,1,164,1,0,0,0,0,0,0,0,0,0,0,0,30,50,0,0,0,0,0,2,0,1,0,0,-1,-1";

    const FIELD_ORDER: [&str; COLUMN_COUNT] = [
        "stage_unlock_requirement",
        "purchase_cost",
        "upgrade_cost_1",
        "upgrade_cost_2",
        "upgrade_cost_3",
        "upgrade_cost_4",
        "upgrade_cost_5",
        "upgrade_cost_6",
        "upgrade_cost_7",
        "upgrade_cost_8",
        "upgrade_cost_9",
        "upgrade_cost_10",
        "currency_type",
        "rarity",
        "guide_order",
        "chapter_unlock_requirement",
        "sell_xp_yield",
        "fanfare_tier",
        "level_cap_ch2",
        "base_max_plus_level",
        "true_form_obtain_level",
        "evolved_form_unlock_level",
        "level_cap_ch1",
        "true_form_id",
        "ultra_form_id",
        "true_form_unlock_level",
        "ultra_form_unlock_level",
        "true_form_xp_cost",
        "true_form_material_1_id",
        "true_form_material_1_quantity",
        "true_form_material_2_id",
        "true_form_material_2_quantity",
        "true_form_material_3_id",
        "true_form_material_3_quantity",
        "true_form_material_4_id",
        "true_form_material_4_quantity",
        "true_form_material_5_id",
        "true_form_material_5_quantity",
        "ultra_form_xp_cost",
        "ultra_form_material_1_id",
        "ultra_form_material_1_quantity",
        "ultra_form_material_2_id",
        "ultra_form_material_2_quantity",
        "ultra_form_material_3_id",
        "ultra_form_material_3_quantity",
        "ultra_form_material_4_id",
        "ultra_form_material_4_quantity",
        "ultra_form_material_5_id",
        "ultra_form_material_5_quantity",
        "level_cap_standard",
        "level_cap_catseye",
        "level_cap_plus",
        "normal_evolution_y_offset",
        "evolved_evolution_y_offset",
        "true_evolution_y_offset",
        "ultra_evolution_y_offset",
        "item_powerup_group",
        "version_added",
        "sell_np_yield",
        "true_form_event_id",
        "plus_cap_grows",
        "egg_id_normal",
        "egg_id_evolved",
    ];

    fn parse_one(line: &str) -> UnitBuy {
        let mut rows = UnitBuy::parse(line, None).unwrap();
        rows.remove(&0).unwrap()
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

        assert_eq!(row.stage_unlock_requirement, 0, "stage_unlock_requirement");
        assert_eq!(row.purchase_cost, 0, "purchase_cost");
        assert_eq!(row.upgrade_cost_1, 50, "upgrade_cost_1");
        assert_eq!(row.upgrade_cost_2, 100, "upgrade_cost_2");
        assert_eq!(row.upgrade_cost_3, 3100, "upgrade_cost_3");
        assert_eq!(row.upgrade_cost_4, 7100, "upgrade_cost_4");
        assert_eq!(row.upgrade_cost_5, 12100, "upgrade_cost_5");
        assert_eq!(row.upgrade_cost_6, 18100, "upgrade_cost_6");
        assert_eq!(row.upgrade_cost_7, 25100, "upgrade_cost_7");
        assert_eq!(row.upgrade_cost_8, 33100, "upgrade_cost_8");
        assert_eq!(row.upgrade_cost_9, 42100, "upgrade_cost_9");
        assert_eq!(row.upgrade_cost_10, 52100, "upgrade_cost_10");
        assert_eq!(row.currency_type, 0, "currency_type");
        assert_eq!(row.rarity, 1, "rarity");
        assert_eq!(row.guide_order, 1000150, "guide_order");
        assert_eq!(
            row.chapter_unlock_requirement, 0,
            "chapter_unlock_requirement"
        );
        assert_eq!(row.sell_xp_yield, 999, "sell_xp_yield");
        assert_eq!(row.fanfare_tier, 3, "fanfare_tier");
        assert_eq!(row.level_cap_ch2, 20, "level_cap_ch2");
        assert_eq!(row.base_max_plus_level, 0, "base_max_plus_level");
        assert_eq!(row.true_form_obtain_level, -1, "true_form_obtain_level");
        assert_eq!(
            row.evolved_form_unlock_level, 10,
            "evolved_form_unlock_level"
        );
        assert_eq!(row.level_cap_ch1, 10, "level_cap_ch1");
        assert_eq!(row.true_form_id, 15017, "true_form_id");
        assert_eq!(row.ultra_form_id, 0, "ultra_form_id");
        assert_eq!(row.true_form_unlock_level, 30, "true_form_unlock_level");
        assert_eq!(row.ultra_form_unlock_level, -1, "ultra_form_unlock_level");
        assert_eq!(row.true_form_xp_cost, 500000, "true_form_xp_cost");
        assert_eq!(row.true_form_material_1_id, 31, "true_form_material_1_id");
        assert_eq!(
            row.true_form_material_1_quantity, 3,
            "true_form_material_1_quantity"
        );
        assert_eq!(row.true_form_material_2_id, 32, "true_form_material_2_id");
        assert_eq!(
            row.true_form_material_2_quantity, 3,
            "true_form_material_2_quantity"
        );
        assert_eq!(row.true_form_material_3_id, 34, "true_form_material_3_id");
        assert_eq!(
            row.true_form_material_3_quantity, 3,
            "true_form_material_3_quantity"
        );
        assert_eq!(row.true_form_material_4_id, 43, "true_form_material_4_id");
        assert_eq!(
            row.true_form_material_4_quantity, 1,
            "true_form_material_4_quantity"
        );
        assert_eq!(row.true_form_material_5_id, 164, "true_form_material_5_id");
        assert_eq!(
            row.true_form_material_5_quantity, 1,
            "true_form_material_5_quantity"
        );
        assert_eq!(row.ultra_form_xp_cost, 0, "ultra_form_xp_cost");
        assert_eq!(row.ultra_form_material_1_id, 0, "ultra_form_material_1_id");
        assert_eq!(
            row.ultra_form_material_1_quantity, 0,
            "ultra_form_material_1_quantity"
        );
        assert_eq!(row.ultra_form_material_2_id, 0, "ultra_form_material_2_id");
        assert_eq!(
            row.ultra_form_material_2_quantity, 0,
            "ultra_form_material_2_quantity"
        );
        assert_eq!(row.ultra_form_material_3_id, 0, "ultra_form_material_3_id");
        assert_eq!(
            row.ultra_form_material_3_quantity, 0,
            "ultra_form_material_3_quantity"
        );
        assert_eq!(row.ultra_form_material_4_id, 0, "ultra_form_material_4_id");
        assert_eq!(
            row.ultra_form_material_4_quantity, 0,
            "ultra_form_material_4_quantity"
        );
        assert_eq!(row.ultra_form_material_5_id, 0, "ultra_form_material_5_id");
        assert_eq!(
            row.ultra_form_material_5_quantity, 0,
            "ultra_form_material_5_quantity"
        );
        assert_eq!(row.level_cap_standard, 30, "level_cap_standard");
        assert_eq!(row.level_cap_catseye, 50, "level_cap_catseye");
        assert_eq!(row.level_cap_plus, 0, "level_cap_plus");
        assert_eq!(
            row.normal_evolution_y_offset, 0,
            "normal_evolution_y_offset"
        );
        assert_eq!(
            row.evolved_evolution_y_offset, 0,
            "evolved_evolution_y_offset"
        );
        assert_eq!(row.true_evolution_y_offset, 0, "true_evolution_y_offset");
        assert_eq!(row.ultra_evolution_y_offset, 0, "ultra_evolution_y_offset");
        assert_eq!(row.item_powerup_group, 2, "item_powerup_group");
        assert_eq!(row.version_added, 0, "version_added");
        assert_eq!(row.sell_np_yield, 1, "sell_np_yield");
        assert_eq!(row.true_form_event_id, 0, "true_form_event_id");
        assert_eq!(row.plus_cap_grows, 0, "plus_cap_grows");
        assert_eq!(row.egg_id_normal, -1, "egg_id_normal");
        assert_eq!(row.egg_id_evolved, -1, "egg_id_evolved");
        assert!(row.rest.is_empty());
    }

    #[test]
    fn the_assembled_views_agree_with_the_raw_columns() {
        let row = parse_one(REAL_ROW);

        assert_eq!(
            row.upgrade_costs(),
            [
                50, 100, 3100, 7100, 12100, 18100, 25100, 33100, 42100, 52100
            ]
        );

        let occupied: Vec<(i32, i32)> = row
            .true_form_materials()
            .map(|material| (material.item_id, material.quantity))
            .collect();
        assert_eq!(occupied, [(31, 3), (32, 3), (34, 3), (43, 1), (164, 1)]);

        assert_eq!(row.ultra_form_materials().count(), 0);
        assert_eq!(row.ultra_form_material_slots().len(), 5);
    }

    #[test]
    fn trailing_columns_keep_their_positions() {
        let mut columns = vec!["0"; COLUMN_COUNT];
        columns.extend(["", "42", "x", "-8"]);

        let row = parse_one(&columns.join(","));
        assert_eq!(row.rest, [None, Some(42), None, Some(-8)]);
    }

    #[test]
    fn the_default_row_agrees_with_the_column_table() {
        let row = serde_json::to_value(UnitBuy::default()).unwrap();
        let fields = row.as_object().unwrap();

        for column in UnitBuy::COLUMNS {
            let declared = column.default.parse::<i64>().unwrap();
            let held = fields.get(column.field).and_then(serde_json::Value::as_i64);

            assert_eq!(held, Some(declared), "{}", column.field);
        }
    }

    #[test]
    fn a_short_row_falls_back_to_each_columns_default() {
        let row = parse_one("0,0,50");

        assert_eq!(row.upgrade_cost_1, 50);
        assert_eq!(row.upgrade_cost_2, 0);
        assert_eq!(row.true_form_id, 0);
        assert_eq!(row.true_form_unlock_level, -1);
        assert_eq!(row.egg_id_evolved, -1);
        assert!(row.rest.is_empty());
    }

    #[test]
    fn blank_lines_do_not_shift_unit_identifiers() {
        let rows = UnitBuy::parse("0,100\n\n0,300\n", None).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows.get(&0).map(|row| row.purchase_cost), Some(100));
        assert_eq!(rows.get(&2).map(|row| row.purchase_cost), Some(300));
    }

    #[test]
    fn a_file_with_no_rows_is_an_error() {
        assert_eq!(
            UnitBuy::parse("\n\n", None).unwrap_err(),
            UnitBuyError::EmptyFile
        );
    }
}
