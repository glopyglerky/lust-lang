// expect: L0306
// rustc E0597: `Q: Ta<Out = A>` with `Q = S` makes `A` the impl's `&'static u32`.
trait Ta { type Out; fn ta(&self, a: Self::Out) -> u32; }
struct S;
impl Ta for S { type Out = &'static u32; fn ta(&self, a: &'static u32) -> u32 { *a } }
fn gk<Q: Ta<Out = A>, A>(q: &Q, a: A) -> u32 { q.ta(a) }
fn main() { let m = 4u32; println!("{}", gk(&S, &m)); }
