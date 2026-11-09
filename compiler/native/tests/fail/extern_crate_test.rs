// expect: L0506
// rustc E0658: the `test` crate needs #![feature(test)].
extern crate test;

fn main() {
    println!("x");
}
