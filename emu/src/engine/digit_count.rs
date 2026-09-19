pub fn digit_count(value: i32) -> i32 {
    let mut rest = value.unsigned_abs();
    let mut digits = 0;

    loop {
        let next = rest / 10;

        digits += 1;

        if rest <= 9 {
            return digits;
        }

        rest = next;
    }
}
