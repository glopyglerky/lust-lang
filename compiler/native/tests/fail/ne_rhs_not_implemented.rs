// expect: L0112
// rustc E0308: only PartialEq<u8> is written for P.
struct P(u8);
impl PartialEq<u8> for P { fn eq(&self, o: &u8) -> bool { self.0 == *o } }
fn main() { println!("{}", P(1) != P(2)); }
