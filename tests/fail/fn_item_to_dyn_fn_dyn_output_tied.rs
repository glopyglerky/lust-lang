// expect: L0733
// rustc "not general enough": the fn item's object lifetime is its
// input's, the target's is 'static (R06 pairing keeps that apart).
trait Tr { fn n(&self) -> i32; }
struct B<'a>(&'a i32);
impl Tr for B<'_> { fn n(&self) -> i32 { *self.0 } }
fn g4<'a>(x: &'a i32) -> Box<dyn Tr + 'a> { Box::new(B(x)) }
fn call(f: &dyn Fn(&i32) -> Box<dyn Tr>) -> i32 { let x = 5; f(&x).n() }
fn main() { println!("{}", call(&g4)); }
