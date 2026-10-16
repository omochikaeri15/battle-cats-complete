//! The engine's pseudo random number generator.
//!
//! Every roll the battle makes, from a critical hit to where a background
//! particle lands, draws from one thirty-two bit xorshift state. A sequence is
//! therefore fully decided by the state it starts from, so a consumer that
//! seeds the generator the same way reproduces the engine's rolls exactly.

use serde::{Deserialize, Serialize};

/// The state the generator falls back to in place of a zero seed.
///
/// The engine itself reseeds from the platform's entropy whenever its state
/// reaches zero, which this crate has no access to, so a zero seed takes a
/// fixed replacement instead.
const REPLACEMENT_SEED: u32 = 1;

/// The engine's xorshift generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rng {
    state: u32,
}

impl Rng {
    /// Creates a generator from a seed.
    ///
    /// # Arguments
    /// * `seed` - The initial state, where zero is replaced by one because a zero state never advances.
    ///
    /// # Returns
    /// A `Rng` positioned to make its first roll from the seed.
    pub fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { REPLACEMENT_SEED } else { seed } }
    }

    /// Returns the current state, from which the sequence can be resumed.
    ///
    /// # Returns
    /// A `u32` holding the state the next roll advances from.
    pub fn state(&self) -> u32 {
        self.state
    }

    /// Advances the generator and reduces the draw into a range.
    ///
    /// A bound of zero or less makes no draw at all and yields zero, which is
    /// how the engine treats a roll that cannot pick anything.
    ///
    /// # Arguments
    /// * `bound` - The exclusive upper limit of the result.
    ///
    /// # Returns
    /// An `i32` in the range from zero up to but excluding the bound.
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
