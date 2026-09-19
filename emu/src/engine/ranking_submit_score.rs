use crate::Fault;

use super::{ranking_entry_id, ranking_submit, AppContext};

pub fn ranking_submit_score(ctx: &mut AppContext, id: i32, score: i32) -> Result<(), Fault> {
    for entry in 0..ctx.ranking_entries.len() {
        if ranking_entry_id(&ctx.ranking_entries[entry]) == id {
            return ranking_submit(ctx, entry, score);
        }
    }

    Ok(())
}
