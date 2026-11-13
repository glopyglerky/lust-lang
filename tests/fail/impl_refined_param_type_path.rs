// expect: L0306
// rustc E0597: `X::put` names the trait method, so the call takes the
// trait's `&'static str` parameter, not the impl's `&str`.
trait Tr { fn put(&mut self, s: &'static str) -> usize; }
struct X { n: usize }
impl Tr for X { fn put(&mut self, s: &str) -> usize { self.n += s.len(); self.n } }
fn main() {
    let mut x = X { n: 0 };
    let t = String::from("t");
    let k = X::put(&mut x, &t);
    println!("{k}");
}
