// expect: L0104
// rustc E0599: `RangeInclusive<u32>` has no `len`.
fn main() {
    println!("{}", (0u32..=5).len());
}
