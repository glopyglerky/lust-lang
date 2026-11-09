// A bound that selects an impl with `'static` in its header accepts `'static` data, and an
// impl that ties two regions accepts two borrows that both live long enough.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
struct S;
struct R<'r>(&'r u32);
impl Tk<&'static u32> for S { fn tk(&self, a: &'static u32) -> u32 { *a } }
impl<'x> Tk<&'x u32> for R<'x> { fn tk(&self, a: &'x u32) -> u32 { *self.0 + *a } }
impl Tk<u32> for &'static u32 { fn tk(&self, a: u32) -> u32 { **self + a } }
trait Ta { type Out; fn ta(&self, a: Self::Out) -> u32; }
impl Ta for S { type Out = &'static u32; fn ta(&self, a: &'static u32) -> u32 { *a + 1 } }
fn gk<Q: Tk<A>, A>(q: &Q, a: A) -> u32 { q.tk(a) + 100 }
fn ga<Q: Ta<Out = A>, A>(q: &Q, a: A) -> u32 { q.ta(a) }
fn mid<Q: Tk<&'static u32>>(q: &Q) -> u32 { gk(q, &5) }
static M: u32 = 9;
fn main() {
    let a = 3u32;
    let b = 4u32;
    let r = R(&a);
    let s: &'static u32 = &M;
    println!("{} {} {}", gk(&S, &M), gk(&r, &b), gk(&s, 2));
    println!("{} {}", ga(&S, &7), mid(&S));
    let d: &dyn Tk<u32> = &s;
    println!("{}", d.tk(1));
}
