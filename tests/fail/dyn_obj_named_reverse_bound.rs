// expect: L0733
// rustc: lifetime may not live long enough. `'b: 'a` is the wrong way
// for returning `dyn T + 'a` as `dyn T + 'b` (fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a, 'b: 'a>(x: &'a (dyn T + 'a)) -> &'a (dyn T + 'b) { x } fn main() { let n = 1u32; println!("{}", f(&n).r()); }
