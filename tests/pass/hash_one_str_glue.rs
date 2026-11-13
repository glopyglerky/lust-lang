// `BuildHasher::hash_one` on a `str` key through generic code: the
// hash glue body writes the key's bytes through a `&[u8]` local,
// whose slice type the glue's C unit must declare (it did not, and
// tcc failed with `Sl_sl_u8_` undeclared).
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

fn one<S: BuildHasher>(s: &S, k: &str) -> u64 {
    s.hash_one(k)
}

fn main() {
    let s = RandomState::new();
    println!("{} {}", one(&s, "hi") == one(&s, "hi"), one(&s, "hi") == one(&s, "ho"));
}
