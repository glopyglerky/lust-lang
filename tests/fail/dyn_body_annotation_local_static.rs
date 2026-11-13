// expect: L0306
// rustc E0597: the let's object lifetime is inferred, so it must outlive the `'static` the
// closure needs, but the value borrows a local.
trait P { fn v(&self) -> u32; }
struct W<'w>(&'w u32);
impl P for W<'_> { fn v(&self) -> u32 { *self.0 } }
fn keep(f: Box<dyn Fn() -> u32>) -> u32 { f() }
fn main() {
    let n = 5u32;
    let b: Box<dyn P> = Box::new(W(&n));
    println!("{}", keep(Box::new(move || b.v())));
}
