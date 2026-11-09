// Same object lifetime on both sides (fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
fn f<'a>(v: &mut Vec<&'a (dyn T + 'a)>, x: &'a (dyn T + 'a)) { v.push(x) } fn main() { let n = 1u32; let mut v: Vec<&dyn T> = Vec::new(); f(&mut v, &n); println!("{}", v[0].r()); }
