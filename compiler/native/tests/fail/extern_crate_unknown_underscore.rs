// expect: L0506
// rustc E0463 also for an unused rename to `_`.
extern crate rand as _;

fn main() {
    println!("x");
}
