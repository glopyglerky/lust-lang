// `xxhash_rust::xxh3`: one-shot XXH3 hashes and the `Xxh3` streaming
// hasher. The streaming hasher keeps its input and hashes all of it at
// `digest`: XXH3 defines the streaming digest as the one-shot hash of
// the same bytes with the same seed, so the values are upstream's. Not
// here (they reject at the use): the `_with_secret` forms, `Xxh3Default`,
// `Xxh3Builder`, `Xxh3::with_secret`, and the `Hasher` impl.

/// XXH3 64-bit of `input`.
pub fn xxh3_64(input: &[u8]) -> u64 {
    core::intrinsics::xxh3_64(input, 0)
}

/// XXH3 64-bit of `input` with a seed.
pub fn xxh3_64_with_seed(input: &[u8], seed: u64) -> u64 {
    core::intrinsics::xxh3_64(input, seed)
}

/// XXH3 128-bit of `input`.
pub fn xxh3_128(input: &[u8]) -> u128 {
    core::intrinsics::xxh3_128(input, 0)
}

/// XXH3 128-bit of `input` with a seed.
pub fn xxh3_128_with_seed(input: &[u8], seed: u64) -> u128 {
    core::intrinsics::xxh3_128(input, seed)
}

/// A streaming XXH3 hasher: `update` any number of times, `digest` or
/// `digest128` at any point (the hasher stays usable), `reset` to start
/// over with the same seed.
#[derive(Clone)]
pub struct Xxh3 {
    seed: u64,
    input: Vec<u8>,
}

impl Xxh3 {
    pub fn new() -> Self {
        Xxh3 { seed: 0, input: Vec::new() }
    }

    pub fn with_seed(seed: u64) -> Self {
        Xxh3 { seed, input: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.input.clear();
    }

    pub fn update(&mut self, input: &[u8]) {
        self.input.extend_from_slice(input);
    }

    pub fn digest(&self) -> u64 {
        core::intrinsics::xxh3_64(self.input.as_slice(), self.seed)
    }

    pub fn digest128(&self) -> u128 {
        core::intrinsics::xxh3_128(self.input.as_slice(), self.seed)
    }
}

impl Default for Xxh3 {
    fn default() -> Self {
        Self::new()
    }
}
