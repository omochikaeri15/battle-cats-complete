use crate::Fault;

use super::{AppContext, draw_worker_cat};

pub fn game_update_lambda_4(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_worker_cat(ctx)
}
