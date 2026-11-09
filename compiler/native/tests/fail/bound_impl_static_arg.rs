// expect: L0306
// rustc E0597: `gk`'s bound `Q: Tk<A>` picks `impl Tk<&'static u32> for S`, so `A` is
// `&'static u32`, but `a` borrows a local.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
struct S;
impl Tk<&'static u32> for S { fn tk(&self, a: &'static u32) -> u32 { *a } }
fn gk<Q: Tk<A>, A>(q: &Q, a: A) -> u32 { q.tk(a) + 100 }
fn run() -> u32 {
    let m = 4u32;
    let a: &u32 = &m;
    gk(&S, a)
}
fn main() { println!("{}", run()); }
