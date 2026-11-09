// expect: L0630
// rustc E0080: 128 const stack frames is the most allowed.
const fn rec(n: u8) -> u8 { if n == 0 { 0 } else { 2 + rec(n - 1) } }
static C: u8 = rec(127);
fn main() { println!("{}", C); }
