// expect: L0506
// A glob of a std module that does not exist (rustc E0432).
use std::nosuch::*;

fn main() {
    println!("hi");
}
