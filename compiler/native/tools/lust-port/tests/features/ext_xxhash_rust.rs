// feature: ext.xxhash_rust
// form: xxh3_64, xxh3_128, and the Xxh3 streaming hasher, as the snapshot keys use them
// cargo-dep: xxhash-rust = { version = "=0.8.15", features = ["xxh3"] }
use xxhash_rust::xxh3::{Xxh3, xxh3_64, xxh3_128};
fn main() {
    println!("{:016x} {:032x}", xxh3_64(b"hello"), xxh3_128(b"hello"));
    let mut h = Xxh3::new();
    h.update(b"hel");
    h.update(b"lo");
    println!("{:016x} {:032x}", h.digest(), h.digest128());
}
