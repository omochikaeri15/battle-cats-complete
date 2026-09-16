use super::{get_cell, AssetStream};

pub fn read_csv_cell(stm: &AssetStream<'_>, col: i32) -> i64 {
    let Some(cell) = get_cell(stm, col) else {
        return 0;
    };

    let text = &stm.bytes[cell.at..];
    let mut at = 0;

    while text.get(at).is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')) {
        at += 1;
    }

    let negative = match text.get(at) {
        Some(b'-') => {
            at += 1;
            true
        }
        Some(b'+') => {
            at += 1;
            false
        }
        _ => false,
    };

    let mut magnitude: u64 = 0;
    let mut digits = 0;

    while let Some(byte) = text.get(at).filter(|byte| byte.is_ascii_digit()) {
        magnitude = magnitude.saturating_mul(10).saturating_add((byte - b'0') as u64);
        digits += 1;
        at += 1;
    }

    if digits == 0 {
        return 0;
    }

    if negative {
        if magnitude >= 1u64 << 63 {
            return i64::MIN;
        }

        return -(magnitude as i64);
    }

    if magnitude > i64::MAX as u64 {
        return i64::MAX;
    }

    magnitude as i64
}
