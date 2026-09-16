pub fn rng_core(seed: &mut u32, bound: i32) -> i32 {
    if bound <= 0 {
        return 0;
    }

    let mut x = *seed;

    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 15;

    *seed = x;

    (x % bound as u32) as i32
}
