// expect: L0506
// A std module that does not exist (rustc E0432).
use std::nosuchmod::Foo;

fn main() {
    println!("hi");
}
