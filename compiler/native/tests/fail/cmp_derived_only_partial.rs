// expect: L0104
// rustc E0599: cmp needs Ord; only PartialOrd is derived.
#[derive(PartialEq, PartialOrd)]
struct P(u8);
fn main() { println!("{:?}", P(1).cmp(&P(2))); }
