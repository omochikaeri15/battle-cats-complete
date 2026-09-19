use crate::Fault;

use super::{draw_worker_cat, AppContext};

pub fn game_update_lambda_4(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_worker_cat(ctx)
}
