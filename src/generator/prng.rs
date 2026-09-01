//! Deterministic fast PRNG (SplitMix64) for reproducible level generation.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FastRng {
    state: u64,
}

impl FastRng {
    /// Initialize with a deterministic 64-bit seed.
    pub fn seed(seed: u64) -> Self {
        let mut s = seed.wrapping_add(0x9E3779B97F4A7C15);
        if s == 0 {
            s = 0x853C49E6748FEA9B;
        }
        Self { state: s }
    }

    /// Advance the PRNG state and return the next pseudo-random 64-bit integer.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Generate an integer in `[start, end)`. If `start >= end`, returns `start`.
    pub fn gen_range(&mut self, start: u32, end: u32) -> u32 {
        if start >= end {
            return start;
        }
        let span = (end - start) as u64;
        start + (self.next_u64() % span) as u32
    }

    /// Return true with the given probability (0.0 to 1.0).
    pub fn gen_bool(&mut self, probability: f32) -> bool {
        let val = (self.next_u64() % 10_000) as f32 / 10_000.0;
        val < probability
    }

    /// Choose a random element from a slice.
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let idx = (self.next_u64() % slice.len() as u64) as usize;
            Some(&slice[idx])
        }
    }

    /// In-place Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        if slice.len() <= 1 {
            return;
        }
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() % (i + 1) as u64) as usize;
            slice.swap(i, j);
        }
    }
}
