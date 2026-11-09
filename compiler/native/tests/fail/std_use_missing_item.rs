// expect: L0506
// An unused import of a std path that does not exist is an error
// (rustc E0432), even with no use of the name.
use std::task::Nonexistent;

fn main() {
    println!("hi");
}
