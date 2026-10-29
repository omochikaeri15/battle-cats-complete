use std::sync::Arc;

use nyanko::chapter::treasure::{data_file, label_file, name_file, set_file, TreasureData, TreasureText, Treasures, SLOTS};

use crate::common::region;
use crate::systems::treasure::Catalog;
use crate::Vfs;

use super::Slot;

const DATA_PREFIX: &str = "treasureData";
const TEXT_PREFIXES: [&str; 3] = ["Treasure1_", "Treasure2", "Treasure3_"];

#[derive(Default)]
pub struct TreasureStore {
    catalog: Slot<Catalog>,
}

impl TreasureStore {
    pub fn catalog(&self, vfs: &Vfs) -> Arc<Catalog> {
        super::cached(&self.catalog, || {
            Catalog::new(
                Treasures {
                    slots: std::array::from_fn(|slot| {
                        data_file(slot).and_then(|file| super::parsed(vfs, &file, TreasureData::parse))
                    }),
                },
                text_slots(vfs, name_file),
                text_slots(vfs, set_file),
                text(vfs, &unsuffixed(&label_file(""))),
            )
        })
    }

    pub(super) fn evict(&self, filename: &str) {
        if filename.starts_with(DATA_PREFIX) || TEXT_PREFIXES.iter().any(|prefix| filename.starts_with(prefix)) {
            super::reset(&self.catalog);
        }
    }

    pub(super) fn clear(&self) {
        super::reset(&self.catalog);
    }
}

fn text_slots(vfs: &Vfs, file: fn(usize, &str) -> Option<String>) -> [Option<TreasureText>; SLOTS] {
    std::array::from_fn(|slot| file(slot, "").and_then(|name| text(vfs, &unsuffixed(&name))))
}

fn text(vfs: &Vfs, filename: &str) -> Option<TreasureText> {
    let mut merged: Option<TreasureText> = None;

    for (name, bytes) in super::named(vfs, filename) {
        let Ok(layer) = TreasureText::parse(bytes, Some(region::text_separator(&name))) else {
            continue;
        };

        let Some(held) = merged.as_mut() else {
            merged = Some(layer);
            continue;
        };

        if layer.rows.len() > held.rows.len() {
            held.rows.resize_with(layer.rows.len(), Vec::new);
        }

        for (row, cells) in held.rows.iter_mut().zip(layer.rows) {
            if row.iter().all(String::is_empty) {
                *row = cells;
            }
        }
    }

    merged
}

fn unsuffixed(name: &str) -> String {
    name.replacen("_.csv", ".csv", 1)
}
