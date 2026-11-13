// expect: L0301
// rustc E0382: max takes self by value.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct S(u8);
fn main() { let a = S(1); let b = a.max(S(2)); println!("{:?} {:?}", a, b); }
