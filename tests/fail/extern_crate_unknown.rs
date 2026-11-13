// expect: L0506
// rustc E0463: no crate named `rand` in this build.
extern crate rand;

fn main() {
    println!("x");
}
