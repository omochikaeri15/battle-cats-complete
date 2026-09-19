use super::NyancomboRecord;

pub fn set_combo_banner_pending(record: &mut NyancomboRecord, pending: u8) {
    record.banner_pending = pending;
}
