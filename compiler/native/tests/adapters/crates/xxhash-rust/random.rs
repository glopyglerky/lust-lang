// xxhash-rust 0.8.15 xxh3 on seeded random inputs up to 5000 bytes:
// one-shot and streaming (random chunk splits) must agree with the real
// crate. Digests fold into checksums; a few lines print in full.
use xxhash_rust::xxh3::{Xxh3, xxh3_128_with_seed, xxh3_64, xxh3_64_with_seed};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn main() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut buf = Vec::new();
    for _ in 0..5000 {
        buf.push(0u8);
    }
    let (mut a, mut b, mut c, mut d) = (0u64, 0u64, 0u128, 0u64);
    for i in 0..600u64 {
        let len = if i < 260 { i as usize } else { (rng.next() % 5000) as usize };
        for j in 0..len {
            buf[j] = rng.next() as u8;
        }
        let seed = if i % 3 == 0 { 0 } else { rng.next() };
        let s = &buf[..len];
        a = a.wrapping_mul(31).wrapping_add(xxh3_64(s));
        b = b.wrapping_mul(31).wrapping_add(xxh3_64_with_seed(s, seed));
        c = c.wrapping_mul(31).wrapping_add(xxh3_128_with_seed(s, seed));
        let mut h = Xxh3::with_seed(seed);
        let mut at = 0;
        while at < len {
            let n = ((rng.next() % 300) as usize).min(len - at);
            h.update(&s[at..at + n]);
            at += n;
        }
        d = d.wrapping_mul(31).wrapping_add(h.digest() ^ (h.digest128() as u64));
        if i % 150 == 7 {
            println!("{} {:016x} {:032x}", len, xxh3_64_with_seed(s, seed), xxh3_128_with_seed(s, seed));
        }
    }
    println!("{:016x} {:016x} {:032x} {:016x}", a, b, c, d);
}
