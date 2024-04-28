#[derive(Default)]
pub struct Stats(pub usize);

impl Stats {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn get_phase_count(&self) -> (i8, i8) {
        ((self.0 % 2 + 1) as i8, 2)
    }
    pub fn get_count(&self) -> usize {
        self.0 / 2
    }
}

