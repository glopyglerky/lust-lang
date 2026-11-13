//! Bench stand-in for the `xxhash_rust` crate (`xxh3` feature), mounted as
//! `crate::xxhash_rust`. Same API as the calls ts-rust makes; the hash is
//! FNV-1a, not XXH3, so values differ from the real crate. lust and rustc
//! compile the same stand-in, so their outputs still agree.

/// `xxhash_rust::xxh3`: one-shot hashes and the streaming `Xxh3` hasher.
pub mod xxh3 {
    fn fnv(input: &[u8], seed: u64) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ seed;
        for &b in input {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    pub fn xxh3_64(input: &[u8]) -> u64 {
        fnv(input, 0)
    }

    pub fn xxh3_64_with_seed(input: &[u8], seed: u64) -> u64 {
        fnv(input, seed)
    }

    pub fn xxh3_128(input: &[u8]) -> u128 {
        ((fnv(input, 0) as u128) << 64) | fnv(input, 0x9e37_79b9_7f4a_7c15) as u128
    }

    /// Streaming hasher: keeps its input and hashes it all at `digest`.
    #[derive(Clone, Default)]
    pub struct Xxh3 {
        input: Vec<u8>,
    }

    impl Xxh3 {
        pub fn new() -> Self {
            Xxh3 { input: Vec::new() }
        }

        pub fn reset(&mut self) {
            self.input.clear();
        }

        pub fn update(&mut self, input: &[u8]) {
            self.input.extend_from_slice(input);
        }

        pub fn digest(&self) -> u64 {
            fnv(&self.input, 0)
        }

        pub fn digest128(&self) -> u128 {
            xxh3_128(&self.input)
        }
    }
}
