use crate::Fault;

use super::AppContext;

pub fn mission_progress(
    ctx: &mut AppContext,
    kind: i32,
    target: i32,
    amount: i32,
    first: i32,
    second: i32,
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "mission_progress",
        })?
        .mission_progress(kind, target, amount, first, second);

    Ok(())
}
