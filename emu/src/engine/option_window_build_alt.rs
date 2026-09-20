use crate::Fault;

use super::AppContext;

pub fn option_window_build_alt(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui()
        .ok_or(Fault::host_missing())?
        .option_window_build(1);

    Ok(())
}
