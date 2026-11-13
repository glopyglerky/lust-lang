// expect: L0104
// rustc E0599: `Neg` is not implemented for unsigned integers.
use std::ops::Neg;

fn main() {
    println!("{}", 5u8.neg());
}
