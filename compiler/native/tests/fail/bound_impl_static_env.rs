// expect: L0733
// rustc: lifetime may not live long enough. `mid`'s own bound `Q: Tk<&'static u32>` proves the
// call's `Q: Tk<A>`, so `A` (the type of `x`) must be `&'static u32`.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
fn gk<Q: Tk<A>, A>(q: &Q, a: A) -> u32 { q.tk(a) }
fn mid<Q: Tk<&'static u32>>(q: &Q, x: &u32) -> u32 { gk(q, x) }
struct S;
impl Tk<&'static u32> for S { fn tk(&self, a: &'static u32) -> u32 { *a } }
static M: u32 = 3;
fn main() { println!("{}", mid(&S, &M)); }
