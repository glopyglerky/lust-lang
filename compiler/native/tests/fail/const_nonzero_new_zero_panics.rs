// expect: L0630
// rustc E0080: `NonZeroU32::new(0)` is `None`, so the const fn panics
// during const evaluation.
use std::num::NonZeroU32;
const fn nz(v: u32) -> NonZeroU32 {
    match NonZeroU32::new(v) {
        Some(n) => n,
        None => panic!("zero"),
    }
}
const Z: NonZeroU32 = nz(0);
fn main() {
    println!("{Z}");
}
