// expect: L0110
// A glob of Ordering's variants brings the variants into patterns.
use std::cmp::Ordering::*;
fn main() {
    let x = 5;
    match x {
        Less => println!("a"),
        _ => println!("b"),
    }
}
