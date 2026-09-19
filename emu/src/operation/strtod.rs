pub fn strtod(text: &[u8]) -> Option<(f64, bool)> {
    let mut at = 0;

    while text
        .get(at)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
    {
        at += 1;
    }

    let start = at;

    if matches!(text.get(at), Some(b'-') | Some(b'+')) {
        at += 1;
    }

    let mut digits = 0;
    let mut nonzero = false;

    while let Some(byte) = text.get(at).filter(|byte| byte.is_ascii_digit()) {
        nonzero |= *byte != b'0';
        digits += 1;
        at += 1;
    }

    if text.get(at) == Some(&b'.') {
        at += 1;

        while let Some(byte) = text.get(at).filter(|byte| byte.is_ascii_digit()) {
            nonzero |= *byte != b'0';
            digits += 1;
            at += 1;
        }
    }

    if digits == 0 {
        return None;
    }

    if matches!(text.get(at), Some(b'e') | Some(b'E')) {
        let mut exponent = at + 1;

        if matches!(text.get(exponent), Some(b'-') | Some(b'+')) {
            exponent += 1;
        }

        if text.get(exponent).is_some_and(|byte| byte.is_ascii_digit()) {
            at = exponent;

            while text.get(at).is_some_and(|byte| byte.is_ascii_digit()) {
                at += 1;
            }
        }
    }

    let value = String::from_utf8_lossy(&text[start..at])
        .parse::<f64>()
        .ok()?;
    let range = value.is_infinite() || (nonzero && (value == 0.0 || value.is_subnormal()));

    Some((value, range))
}
