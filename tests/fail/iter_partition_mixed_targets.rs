// expect: L0110
// rustc E0308: `partition` answers `(B, B)`, one collection type for both sides.
use std::collections::VecDeque;
fn main() {
    let (a, b): (VecDeque<u8>, Vec<u8>) = (0..6u8).partition(|x| x % 2 == 0);
    println!("{:?} {:?}", a, b);
}
