// expect: L0506
// `{self}` needs a module or an enum: `Rc` is a struct (rustc E0432).
use std::rc::Rc::{self};

fn main() {
    println!("hi");
}
