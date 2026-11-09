// expect: L0101
// rustc E0433: `mem` is not in scope without `use std::mem`.
fn main() {
    println!("{}", mem::size_of_val(&1u8));
}
