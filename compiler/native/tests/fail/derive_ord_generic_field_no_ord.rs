// expect: L0104
// rustc E0599: derived Ord bounds T: Ord; the argument type is not Ord.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct W<T>(T);
#[derive(PartialEq, Eq, PartialOrd)]
struct N(u8);
fn main() { println!("{:?}", W(N(1)).cmp(&W(N(2)))); }
