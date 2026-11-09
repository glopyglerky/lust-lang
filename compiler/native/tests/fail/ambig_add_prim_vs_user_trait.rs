// expect: L0720
// rustc E0034: `Add::add` and a user trait `add` both apply to `u8`.
use std::ops::Add;

trait Weird {
    fn add(self, rhs: Self) -> Self;
}

impl Weird for u8 {
    fn add(self, _rhs: u8) -> u8 {
        99
    }
}

fn main() {
    println!("{}", 1u8.add(2u8));
}
