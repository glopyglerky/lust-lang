// rustc-error: E0277
// lust-expect: L0110
#[derive(Eq)]
struct P(u8);
fn main() { println!("x"); }
