// `&'a (dyn T + 'b)` implies `'b: 'a`, so it pushes into a Vec of
// `&'a (dyn T + 'a)` (fx-dyn-review's case; rustc accepts it).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a, 'b>(v: &mut Vec<&'a (dyn T + 'a)>, x: &'a (dyn T + 'b)) { v.push(x) } fn main() { let n = 1u32; let mut v: Vec<&dyn T> = Vec::new(); f(&mut v, &n); println!("{}", v[0].r()); }
