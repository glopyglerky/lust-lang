// expect: L0001
// rustc: visibility `pub(crate)` is not followed by an item (at the end of the
// file; fuzz seed 235).
fn main() {
    println!("x");
}
pub(crate)
