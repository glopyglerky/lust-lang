// expect: L0306
// rustc E0597: `W<S>: Tk<A>` needs `S: In<A>` (the impl's where-clause), which picks
// `impl In<&'static u32> for S`; the borrow of a local is not `'static`.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
trait In<A> { fn inn(&self, a: A) -> u32; }
struct S;
impl In<&'static u32> for S { fn inn(&self, a: &'static u32) -> u32 { *a } }
struct W<Q>(Q);
impl<Q: In<A>, A> Tk<A> for W<Q> { fn tk(&self, a: A) -> u32 { self.0.inn(a) } }
fn gk<Q: Tk<A>, A>(q: &Q, a: A) -> u32 { q.tk(a) + 100 }
fn main() { let m = 4u32; println!("{}", gk(&W(S), &m)); }
