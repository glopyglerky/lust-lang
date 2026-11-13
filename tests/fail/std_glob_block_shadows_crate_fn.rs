// expect: L0505
// rustc E0277: the block's `std::iter::*` brings `zip`, which shadows the
// crate's `zip`, and i32 is not an iterator. Lust does not know every
// name a std glob brings, so it rejects the name.
fn zip(a: i32, b: i32) -> i32 { a * 10 + b }
fn main() {
    use std::iter::*;
    println!("{}", zip(1, 2));
}
