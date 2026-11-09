// A `'static` object shortens to `'a` (fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a>(x: &'a (dyn T + 'static)) -> &'a (dyn T + 'a) { x } fn main() { static N: u32 = 1; println!("{}", f(&N).r()); }
