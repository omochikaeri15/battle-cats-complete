use crate::Fault;

use super::{ranking_entry_id, AppContext};

pub fn ranking_submit(ctx: &mut AppContext, entry: usize, score: i32) -> Result<(), Fault> {
    let id = ctx.ranking_entries.get(entry).map_or(0, ranking_entry_id);

    ctx.platform().ok_or(Fault::HostMissing { site: "ranking_submit" })?.ranking_submit(id, score);

    Ok(())
}
