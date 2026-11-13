// expect: L0506
// `self` of a std module that does not exist (rustc E0432).
use std::nosuch::{self};

fn main() {
    println!("hi");
}
