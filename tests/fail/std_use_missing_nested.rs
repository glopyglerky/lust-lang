// expect: L0506
// A missing leaf in a nested use tree under std (rustc E0432).
use std::{fmt, nope};

fn main() {
    let _ = fmt::Error;
    println!("hi");
}
