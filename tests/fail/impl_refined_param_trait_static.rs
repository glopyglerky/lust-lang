// expect: L0306
// rustc E0597: the impl takes `&str` where the trait takes `&'static
// str` (allowed), but callers see the trait's signature.
trait Tr { fn put(&mut self, s: &'static str) -> usize; }
struct X { n: usize }
impl Tr for X { fn put(&mut self, s: &str) -> usize { self.n += s.len(); self.n } }
fn main() {
    let mut x = X { n: 0 };
    let t = String::from("t");
    let k = x.put(&t);
    println!("{k}");
}
