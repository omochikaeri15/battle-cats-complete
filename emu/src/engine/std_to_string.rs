pub fn std_to_string(value: i32) -> Vec<u8> {
    let mut digits = [0u8; 11];
    let negative = value < 0;
    let mut rest = value as i64;

    if negative {
        rest = -rest;
    }

    let mut at = digits.len();

    loop {
        at -= 1;
        digits[at] = b'0' + (rest % 10) as u8;
        rest /= 10;

        if rest == 0 {
            break;
        }
    }

    let mut text = Vec::new();

    if negative {
        text.push(b'-');
    }

    text.extend_from_slice(&digits[at..]);
    text
}
