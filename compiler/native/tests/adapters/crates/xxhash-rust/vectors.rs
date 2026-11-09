// xxhash-rust 0.8.15 xxh3 on fixed inputs at every length class the
// algorithm branches on (0, 1-3, 4-8, 9-16, 17-128, 129-240, long, and
// the block and stripe edges), seeded and unseeded, plus Xxh3 streaming.
use xxhash_rust::xxh3::{Xxh3, xxh3_128, xxh3_128_with_seed, xxh3_64, xxh3_64_with_seed};

fn main() {
    let mut data = Vec::new();
    for i in 0..2100u32 {
        data.push((i.wrapping_mul(2654435761) >> 24) as u8);
    }
    let lens = [0usize, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 32, 33, 64, 65, 96, 97, 127, 128, 129, 200, 239, 240, 241, 255, 256, 1023, 1024, 1025, 2048, 2100];
    for len in lens {
        let s = &data[..len];
        println!("{} {:016x} {:016x} {:032x} {:032x}", len, xxh3_64(s), xxh3_64_with_seed(s, 0x9E37_79B9), xxh3_128(s), xxh3_128_with_seed(s, 1u64 << 63));
    }
    println!("{:016x} {:032x}", xxh3_64(b"hello"), xxh3_128(b"hello world"));
    let mut h = Xxh3::new();
    println!("{:016x} {:032x}", h.digest(), h.digest128());
    h.update(b"hel");
    h.update(b"lo");
    println!("{:016x} {:032x}", h.digest(), h.digest128());
    h.update(&data[..1500]);
    println!("{:016x} {:032x}", h.digest(), h.digest128());
    h.reset();
    h.update(b"hello");
    println!("{:016x}", h.digest());
    let mut s = Xxh3::with_seed(42);
    s.update(&data[..300]);
    let copy = s.clone();
    s.update(b"tail");
    println!("{:016x} {:016x} {:032x}", copy.digest(), s.digest(), s.digest128());
    let d: Xxh3 = Default::default();
    println!("{:016x}", d.digest());
}
