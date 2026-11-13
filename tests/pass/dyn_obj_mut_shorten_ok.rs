// rustc lets a coercion shorten the object lifetime of the outermost
// `&mut dyn` (fx-dyn-review's case; rustc accepts it).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a, 'b>(x: &'b mut (dyn T + 'a)) -> &'b mut (dyn T + 'b) { x } fn main() { let mut n = 1u32; println!("{}", f(&mut n).r()); }
