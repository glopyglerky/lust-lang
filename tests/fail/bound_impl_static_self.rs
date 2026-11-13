// expect: L0306
// rustc E0597: the bound `Q: Tk<u32>` picks `impl Tk<u32> for &'static u32`, so `Q` (here
// `&u32`) must be `&'static u32`, but `r` borrows a local.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
impl Tk<u32> for &'static u32 { fn tk(&self, a: u32) -> u32 { **self + a } }
fn gk<Q: Tk<A>, A>(q: &Q, a: A) -> u32 { q.tk(a) + 100 }
fn main() {
    let m = 4u32;
    let r: &u32 = &m;
    println!("{}", gk(&r, 1u32));
}
