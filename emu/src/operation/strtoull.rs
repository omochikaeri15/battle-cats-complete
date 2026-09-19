pub fn strtoull(text: &[u8], base: u32) -> Option<(u64, bool)> {
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
    let mut overflow = false;

    while let Some(digit) = text.get(at).and_then(|byte| (*byte as char).to_digit(36)).filter(|digit| *digit < base) {
        match magnitude.checked_mul(base as u64).and_then(|scaled| scaled.checked_add(digit as u64)) {
            Some(next) => magnitude = next,
            None => {
                magnitude = u64::MAX;
                overflow = true;
            }
        }

        digits += 1;
        at += 1;
    }

    if digits == 0 {
        return None;
    }

    if overflow {
        return Some((u64::MAX, true));
    }

    Some((if negative { magnitude.wrapping_neg() } else { magnitude }, false))
}
