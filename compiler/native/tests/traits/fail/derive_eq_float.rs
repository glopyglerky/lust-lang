// rustc-error: E0277
// lust-expect: L0110
#[derive(PartialEq, Eq)]
struct P(f64);
fn main() { println!("x"); }
