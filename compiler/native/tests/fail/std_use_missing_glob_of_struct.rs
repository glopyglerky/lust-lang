// expect: L0506
// A glob needs a module or an enum: `Vec` is a struct (rustc E0432).
use std::vec::Vec::*;

fn main() {
    println!("hi");
}
