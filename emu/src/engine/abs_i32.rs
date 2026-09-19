pub fn abs_i32(value: i32) -> i32 {
    let negated = value.wrapping_neg();

    if negated < 0 {
        value
    } else {
        negated
    }
}
