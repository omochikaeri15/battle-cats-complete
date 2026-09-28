use std::collections::BTreeSet;

use super::AppContext;

pub fn reset_cannon_part_rows(ctx: &mut AppContext) {
    ctx.cannon_part_rows.clear();

    for (&part, kinds) in &ctx.castle_recipes {
        let mut present: BTreeSet<i32> = BTreeSet::new();

        for &kind in kinds.keys() {
            present.insert(kind);
        }

        if part != 0 {
            ctx.cannon_part_rows.entry(part).or_default().push(0);
        } else {
            ctx.cannon_part_rows.entry(part).or_default().push(3);
        }

        ctx.cannon_part_rows.entry(part).or_default().push(0);

        let mut kind = 1usize;

        while present.len() > kind {
            ctx.cannon_part_rows.entry(part).or_default().push(0);
            kind += 1;
        }
    }
}
