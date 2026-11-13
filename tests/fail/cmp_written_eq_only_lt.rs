// expect: L0112
// rustc E0369: a written PartialEq gives no PartialOrd.
struct P(u8);
impl PartialEq for P { fn eq(&self, o: &P) -> bool { self.0 == o.0 } }
fn main() { println!("{}", P(1) < P(2)); }
