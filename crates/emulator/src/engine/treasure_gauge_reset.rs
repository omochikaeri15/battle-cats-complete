use super::TreasureGauge;

pub fn treasure_gauge_reset(gauge: &mut TreasureGauge) {
    gauge.value = 0;
}
