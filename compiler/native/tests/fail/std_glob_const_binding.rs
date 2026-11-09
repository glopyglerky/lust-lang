// expect: L0505
// rustc E0005: `PI` is the glob's const, so `let PI` is a refutable pattern.
use std::f64::consts::*;
fn main() {
    let PI = 3.0;
    println!("{}", PI);
}
