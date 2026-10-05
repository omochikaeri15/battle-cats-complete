use super::TreasureGauge;

pub fn treasure_gauge_full(gauge: &TreasureGauge) -> bool {
    gauge.value >= gauge.max_value
}
