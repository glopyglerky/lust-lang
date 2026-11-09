// `type D<'a> = &'a dyn T`: the object lifetime defaults to `'a`.
trait T { fn r(&self) -> usize; }
impl T for u32 { fn r(&self) -> usize { *self as usize } }
type D<'a> = &'a dyn T;
fn pick<'a>(a: D<'a>, _b: D<'a>) -> D<'a> { a }
fn main() { let x = 1u32; let y = 2u32; println!("{}", pick(&x, &y).r()); }
