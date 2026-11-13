// expect: L0506
// A missing core item imported under a new name (rustc E0432).
use core::task::Nope as N;

fn main() {
    println!("hi");
}
