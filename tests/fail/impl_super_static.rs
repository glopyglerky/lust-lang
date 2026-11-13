// expect: L0617
// rustc E0308 (lifetime mismatch): `impl<'a> Sup<&'a u32> for S` owes `S: Tk<&'a u32>` for
// every `'a`, and the only `Tk` impl gives it for `'static` alone.
trait Tk<A> {}
struct S;
impl Tk<&'static u32> for S {}
trait Sup<A>: Tk<A> {}
impl<'a> Sup<&'a u32> for S {}
fn main() {
    println!("x");
}
