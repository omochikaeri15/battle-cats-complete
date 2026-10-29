use crate::Fault;

use super::{AppContext, std_chrono_system_clock_now};

pub fn now_seconds(ctx: &mut AppContext) -> Result<f64, Fault> {
    Ok((std_chrono_system_clock_now(ctx)? / 1_000_000) as f64)
}
