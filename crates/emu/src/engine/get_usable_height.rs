use super::ScreenMetrics;

pub fn get_usable_height(metrics: &ScreenMetrics) -> i32 {
    metrics
        .screen_h
        .wrapping_sub(metrics.inset_top.wrapping_add(metrics.inset_bottom))
}
