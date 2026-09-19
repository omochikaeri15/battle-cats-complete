use std::cmp::Ordering;

use crate::Fault;

use super::{
    AppContext, NyancomboRecord, combo_sort_less, combo_unlocked, page_list_set_count,
    page_list_set_page,
};

const SITE: &str = "combo_list_rebuild";

pub fn combo_list_rebuild(
    ctx: &mut AppContext,
    tab: i32,
    reset_page: u8,
    unit: i32,
    revealed_only: u8,
) -> Result<(), Fault> {
    ctx.combo_store.records.clear();

    let mut definition = 0usize;

    while definition < ctx.combo_definitions.len() {
        let kinds = ctx
            .combo_store
            .tab_kinds
            .get(tab as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: tab as i64,
                limit: ctx.combo_store.tab_kinds.len() as i64,
            })?
            .len();
        let mut entry = 0usize;

        while entry < kinds {
            let kind = ctx.combo_store.tab_kinds[tab as i64 as usize][entry];
            let cells = ctx
                .combo_definitions
                .get(definition)
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: definition as i64,
                    limit: ctx.combo_definitions.len() as i64,
                })?;

            if kind
                != *cells.get(13).ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: 13,
                    limit: cells.len() as i64,
                })?
            {
                entry += 1;
                continue;
            }

            ctx.combo_store.records.push(NyancomboRecord::default());

            let cells = &ctx.combo_definitions[definition];
            let record = ctx
                .combo_store
                .records
                .last_mut()
                .ok_or(Fault::NullPointer { site: SITE })?;

            record.unit_count = 5;

            let mut first_empty = 5;

            for (cell, &value) in cells.iter().enumerate() {
                match cell {
                    0 => record.combo_id = value,
                    1 => record.availability = value,
                    2 => record.charagroup_id = value,
                    _ if cell - 3 <= 9 => {
                        let member = (cell - 3) >> 1;

                        if (cell - 3) & 1 != 0 {
                            record.form[member] = value;
                        } else {
                            record.unit[member] = value;

                            if value == -1 && first_empty == 5 {
                                record.unit_count = member as i32;
                                first_empty = member as i32;
                            }
                        }
                    }
                    _ => {
                        let effect = (cell - 13) >> 1;
                        let slot = if (cell - 13) & 1 != 0 {
                            3 + effect
                        } else {
                            effect
                        };
                        let target = match slot {
                            0..=2 => &mut record.kind[slot],
                            3..=5 => &mut record.power[slot - 3],
                            6 => &mut record.effect_count,
                            _ => {
                                return Err(Fault::IndexOutOfRange {
                                    site: SITE,
                                    index: cell as i64,
                                    limit: 19,
                                });
                            }
                        };

                        *target = value;

                        if (cell - 13) & 1 == 0 && value == -1 {
                            record.effect_count = effect as i32;
                        }
                    }
                }
            }

            record.name_index = definition as i32;
            record.state =
                *ctx.combo_store
                    .states
                    .get(definition)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: definition as i64,
                        limit: ctx.combo_store.states.len() as i64,
                    })?;

            let revealed = *ctx
                .combo_store
                .revealed
                .entry(definition as i32)
                .or_insert(0);
            let record = ctx
                .combo_store
                .records
                .last_mut()
                .ok_or(Fault::NullPointer { site: SITE })?;

            record.reveal_badge = revealed;
            record.revealed = revealed;

            let record = ctx
                .combo_store
                .records
                .last()
                .ok_or(Fault::NullPointer { site: SITE })?;

            if !combo_unlocked(ctx, record)? {
                ctx.combo_store.records.pop();
                break;
            }

            if record.state == 1 {
                let record = ctx
                    .combo_store
                    .records
                    .last_mut()
                    .ok_or(Fault::NullPointer { site: SITE })?;

                record.state = 0;
                *ctx.combo_store
                    .states
                    .get_mut(definition)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: definition as i64,
                        limit: 0,
                    })? = 0;
                ctx.combo_store.unlock_notices.insert(definition as i32, 1);
                ctx.combo_store.revealed.insert(definition as i32, 1);

                let record = ctx
                    .combo_store
                    .records
                    .last_mut()
                    .ok_or(Fault::NullPointer { site: SITE })?;

                record.revealed = 1;
                record.reveal_badge = 1;
                ctx.set_block_at::<1>(AppContext::COMBO_UNLOCK_NOTICE, [1])?;
                break;
            }

            if revealed_only != 0
                && *ctx
                    .combo_store
                    .revealed
                    .entry(definition as i32)
                    .or_insert(0)
                    == 0
            {
                ctx.combo_store.records.pop();
                break;
            }

            if unit != -1 {
                let record = ctx
                    .combo_store
                    .records
                    .last()
                    .ok_or(Fault::NullPointer { site: SITE })?;

                if record.unit[1..].iter().all(|&member| member != unit) && record.unit[0] != unit {
                    ctx.combo_store.records.pop();
                }
            }

            entry += 1;
        }

        definition += 1;
    }

    if ctx.combo_store.records.len() >= 2 {
        let mut records = std::mem::take(&mut ctx.combo_store.records);
        let mut fault = None;

        if tab != 0 {
            records.sort_by(|left, right| {
                match (
                    combo_sort_less(ctx, left, right),
                    combo_sort_less(ctx, right, left),
                ) {
                    (Ok(true), _) => Ordering::Less,
                    (Ok(false), Ok(true)) => Ordering::Greater,
                    (Ok(false), Ok(false)) => Ordering::Equal,
                    (Err(error), _) | (_, Err(error)) => {
                        fault.get_or_insert(error);
                        Ordering::Equal
                    }
                }
            });
        } else {
            records.sort_by_key(|record| record.unit_count);
        }

        ctx.combo_store.records = records;

        if let Some(error) = fault {
            return Err(error);
        }
    }

    if reset_page != 0 {
        ctx.combo_store.page = 0;
        page_list_set_page(ctx, AppContext::COMBO_PAGE_LIST, ctx.combo_store.page)?;
    }

    let count = ctx.combo_store.records.len() as i32;
    let last = count.wrapping_sub(1);

    page_list_set_count(
        ctx,
        AppContext::COMBO_PAGE_LIST,
        (if last < 0 {
            count.wrapping_add(2)
        } else {
            last
        } >> 2)
            .wrapping_add(1),
    )
}
