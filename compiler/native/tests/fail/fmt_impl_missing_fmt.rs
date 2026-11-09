// expect: L0110
// rustc E0046: an impl of Display must define `fmt`.
use std::fmt;
struct A;
impl fmt::Display for A {}
fn main() {
    println!("{}", A);
}
