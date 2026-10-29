mod enemyname;
mod enemypicturebook;

pub mod t_unit;

use serde::{Deserialize, Serialize};

use crate::combat::Entity;

pub use enemyname::{EnemyName, EnemyNameError};
pub use enemypicturebook::{EnemyPictureBook, EnemyPictureBookError};

#[derive(Debug, Clone, Copy)]
pub struct Tables<'a> {
    pub combat: &'a [Entity],
    pub names: &'a [EnemyName],
    pub picture_book: &'a [EnemyPictureBook],
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: u32,
    pub name: Option<String>,
    pub description: Option<Vec<String>>,
    pub combat: Option<Entity>,
    pub attack_frames: Option<i32>,
}

impl Unit {
    pub fn assemble(id: u32, attack_frames: Option<i32>, tables: &Tables<'_>) -> Self {
        let index = id as usize;

        Self {
            id,
            name: tables.names.get(index).and_then(|entry| entry.name.clone()),
            description: tables
                .picture_book
                .get(index)
                .and_then(|entry| entry.description.clone()),
            combat: tables.combat.get(index).cloned(),
            attack_frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_indexes_every_table_by_id() {
        let combat = t_unit::parse(
            "header one\nheader two\n100,1,10,50,20,300,75,60,0,120\n400,3,20,90,20,300,75,60,0,120\n",
            None,
        )
        .unwrap();
        let names = EnemyName::parse("Doge\nSnache").unwrap();
        let book = EnemyPictureBook::parse("0,A small dog\n1,A red snake", None).unwrap();

        let tables = Tables { combat: &combat, names: &names, picture_book: &book };

        let doge = Unit::assemble(0, Some(30), &tables);
        assert_eq!(doge.id, 0);
        assert_eq!(doge.name.as_deref(), Some("Doge"));
        assert_eq!(doge.description.as_ref().unwrap()[0], "A small dog");
        assert_eq!(doge.combat.as_ref().unwrap().hitpoints, 100);
        assert_eq!(doge.attack_frames, Some(30));

        let snache = Unit::assemble(1, None, &tables);
        assert_eq!(snache.name.as_deref(), Some("Snache"));
        assert_eq!(snache.combat.as_ref().unwrap().hitpoints, 400);
    }

    #[test]
    fn assemble_leaves_absent_rows_empty() {
        let combat = Vec::new();
        let names = Vec::new();
        let book = Vec::new();
        let tables = Tables { combat: &combat, names: &names, picture_book: &book };

        let unit = Unit::assemble(999, None, &tables);

        assert_eq!(unit.id, 999);
        assert!(unit.name.is_none());
        assert!(unit.description.is_none());
        assert!(unit.combat.is_none());
    }

    #[test]
    fn parse_row_agrees_with_the_whole_table() {
        let raw = "header one\nheader two\n100,1,10,50,20,300,75,60,0,120\n400,3,20,90,20,300,75,60,0,120\n";

        let all = t_unit::parse(raw, None).unwrap();
        assert_eq!(t_unit::parse_row(raw, 1, None).as_ref(), all.get(1));
        assert!(t_unit::parse_row(raw, 99, None).is_none());

        let names = EnemyName::parse("Doge\nSnache").unwrap();
        assert_eq!(EnemyName::parse_row("Doge\nSnache", 1).as_ref(), names.get(1));
        assert!(EnemyName::parse_row("Doge\nSnache", 99).is_none());
    }

    #[test]
    fn placeholder_names_resolve_to_none() {
        let names = EnemyName::parse("Doge\nダミー\n").unwrap();
        assert_eq!(names[0].name.as_deref(), Some("Doge"));
        assert!(names[1].name.is_none());
    }
}
