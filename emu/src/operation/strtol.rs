pub struct Strtol {
    pub value: i64,
    pub end: usize,
    pub overflow: bool,
}

pub fn strtol(text: &[u8], base: i32) -> Strtol {
    if base < 0 || base == 1 || base > 36 {
        return Strtol {
            value: 0,
            end: 0,
            overflow: false,
        };
    }

    let mut at = 0;

    while text
        .get(at)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
    {
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

    let mut radix = base as u64;

    if (base == 0 || base == 16)
        && text.get(at) == Some(&b'0')
        && text
            .get(at + 1)
            .is_some_and(|byte| matches!(byte, b'x' | b'X'))
        && text
            .get(at + 2)
            .is_some_and(|byte| byte.is_ascii_hexdigit())
    {
        at += 2;
        radix = 16;
    }

    if radix == 0 {
        radix = if text.get(at) == Some(&b'0') { 8 } else { 10 };
    }

    let mut magnitude: u64 = 0;
    let mut digits = 0;
    let mut overflow = false;

    while let Some(digit) = text
        .get(at)
        .and_then(|byte| (*byte as char).to_digit(36))
        .map(u64::from)
        .filter(|digit| *digit < radix)
    {
        match magnitude
            .checked_mul(radix)
            .and_then(|scaled| scaled.checked_add(digit))
        {
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
        return Strtol {
            value: 0,
            end: 0,
            overflow: false,
        };
    }

    if negative {
        if magnitude > 1u64 << 63 {
            return Strtol {
                value: i64::MIN,
                end: at,
                overflow: true,
            };
        }

        return Strtol {
            value: (magnitude as i64).wrapping_neg(),
            end: at,
            overflow,
        };
    }

    if magnitude > i64::MAX as u64 {
        return Strtol {
            value: i64::MAX,
            end: at,
            overflow: true,
        };
    }

    Strtol {
        value: magnitude as i64,
        end: at,
        overflow,
    }
}
