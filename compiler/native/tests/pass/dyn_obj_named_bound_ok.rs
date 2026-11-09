// With `'a: 'b`, `&'b (dyn T + 'a)` returns as `&'b (dyn T + 'b)`
// (fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a: 'b, 'b>(x: &'b (dyn T + 'a)) -> &'b (dyn T + 'b) { x } fn main() { let n = 1u32; println!("{}", f(&n).r()); }
