// expect: L0113
// rustc E0308: no PartialEq<u8> impl for P.
struct P(u8);
impl PartialEq for P { fn eq(&self, o: &P) -> bool { self.0 == o.0 } }
fn main() { println!("{}", P(1) == 1u8); }
