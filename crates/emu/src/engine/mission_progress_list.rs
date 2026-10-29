use crate::Fault;

use super::AppContext;

pub fn mission_progress_list(
    ctx: &mut AppContext,
    kind: i32,
    targets: &[i32],
    amount: i32,
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .mission_progress_list(kind, targets, amount);

    Ok(())
}
