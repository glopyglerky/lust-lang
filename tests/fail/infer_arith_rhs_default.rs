// expect: L0111
// rustc E0284: `u8: Shl<?R>` (and `u8: Add<?R>`, with `Rhs = u8` and
// `Rhs = &u8`) does not fix `?R`, so `Default::default()` on the right of
// a known number has no type (hunt6).
fn main() {
    let x = 1u8 << Default::default();
    println!("{}", x);
}
