use crate::Fault;

use super::{std_chrono_system_clock_now, AppContext};

pub fn now_seconds(ctx: &mut AppContext) -> Result<f64, Fault> {
    Ok((std_chrono_system_clock_now(ctx)? / 1_000_000) as f64)
}
