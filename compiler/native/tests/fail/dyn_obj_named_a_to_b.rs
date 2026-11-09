// expect: L0733
// rustc: lifetime may not live long enough. The object lifetime `'a` does
// not outlive `'b` (case from fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a, 'b>(x: &'a (dyn T + 'a)) -> &'a (dyn T + 'b) { x } fn main() { let n = 1u32; println!("{}", f(&n).r()); }
