// expect: L0506
// An empty group still names its module (rustc E0432).
use std::nosuch::{};

fn main() {
    println!("hi");
}
