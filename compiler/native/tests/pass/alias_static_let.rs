// A `let` annotation through a type alias with `'static` references (module and block-local)
// holds values that are `'static`.
trait P { fn v(&self) -> u32; }
struct W<'w>(&'w u32);
impl P for W<'_> { fn v(&self) -> u32 { *self.0 } }
type Alias = Option<&'static u32>;
type B = Box<dyn P>;
static N: u32 = 3;
fn main() {
    type R = &'static u32;
    let x: R = &N;
    let y: Alias = Some(&N);
    let v: Vec<R> = vec![&N, x];
    let b: B = Box::new(W(&N));
    let f = |r: R| *r + 1;
    println!("{} {:?} {:?} {} {}", x, y, v, b.v(), f(&N));
}
