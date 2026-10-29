//! Integer arithmetic the engine performs while reconciling loop lengths.

/// Calculates the greatest common divisor of two integers.
///
/// The result carries the sign the engine's own Euclidean loop produces, which
/// follows the truncating remainder rather than the mathematical convention.
///
/// # Arguments
/// * `first` - The first operand.
/// * `second` - The second operand.
///
/// # Returns
/// An `i32` containing the divisor, which is the non-zero operand when the other
/// is zero and zero when both are.
pub fn gcd(first: i32, second: i32) -> i32 {
    let (mut first, mut second) = (first, second);

    while second != 0 {
        let previous = second;
        second = first.wrapping_rem(second);
        first = previous;
    }

    first
}

/// Calculates the least common multiple of two integers.
///
/// # Arguments
/// * `first` - The first operand.
/// * `second` - The second operand.
///
/// # Returns
/// An `i32` containing the multiple, wrapping on overflow as the engine does,
/// and zero when both operands are zero.
pub fn lcm(first: i32, second: i32) -> i32 {
    let divisor = gcd(first, second);

    if divisor == 0 { return 0; }

    first.wrapping_div(divisor).wrapping_mul(second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_of_common_pairs() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(101, 10), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(5, 0), 5);
        assert_eq!(gcd(0, 0), 0);
    }

    #[test]
    fn lcm_of_common_pairs() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(21, 6), 42);
        assert_eq!(lcm(1, 8), 8);
        assert_eq!(lcm(0, 0), 0);
    }
}
