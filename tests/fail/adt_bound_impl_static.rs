// expect: L0306
// rustc E0597: `W<S, &u32>` is well-formed only if `S: Tk<&u32>`, and the only impl is
// `impl Tk<&'static u32> for S`; `m` is a local.
trait Tk<A> { fn tk(&self, a: A) -> u32; }
struct S;
impl Tk<&'static u32> for S { fn tk(&self, a: &'static u32) -> u32 { *a } }
struct W<Q: Tk<A>, A>(Q, A);
enum E<Q, A> where Q: Tk<A> { V(Q, A), N }
fn main() {
    let m = 4u32;
    let w = W(S, &m);
    let e = E::V(S, &M);
    println!("{} {}", matches!(e, E::N), w.0.tk(&M));
}
static M: u32 = 3;
