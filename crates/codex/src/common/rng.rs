use serde::{Deserialize, Serialize};

const REPLACEMENT_SEED: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { REPLACEMENT_SEED } else { seed } }
    }

    pub fn state(&self) -> u32 {
        self.state
    }

    pub fn roll(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }

        let mut x = self.state;

        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 15;

        self.state = x;

        (x % bound as u32) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn a_zero_bound_draws_nothing() {
        let mut rng = Rng::new(100);
        let before = rng.state();

        assert_eq!(rng.roll(0), 0);
        assert_eq!(rng.state(), before);
    }

    #[test]
    fn the_same_seed_replays_the_same_rolls() {
        let mut first = Rng::new(100);
        let mut second = Rng::new(100);

        let rolled: Vec<i32> = (0..16).map(|_| first.roll(1000)).collect();
        let replayed: Vec<i32> = (0..16).map(|_| second.roll(1000)).collect();

        assert_eq!(rolled, replayed);
    }

    #[test]
    fn a_zero_seed_still_advances() {
        let mut rng = Rng::new(0);

        rng.roll(10);

        assert_ne!(rng.state(), 0);
    }
}
