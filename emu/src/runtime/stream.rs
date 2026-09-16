pub struct Stream {
    state: u32,
}

impl Stream {
    pub fn seeded(seed: u32) -> Option<Self> {
        (seed != 0).then_some(Self { state: seed })
    }

    pub fn state(&self) -> u32 {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut u32 {
        &mut self.state
    }
}
