// expect: L0631
// rustc E0015: f64::sqrt is not a const fn.
const fn h(x: f64) -> f64 { x.sqrt() }
fn main() { println!("{}", h(4.0)); }
