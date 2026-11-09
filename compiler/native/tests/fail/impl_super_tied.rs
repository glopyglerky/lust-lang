// expect: L0617
// rustc: lifetime may not live long enough. The selected `Tk` impl names one lifetime twice,
// so the two lifetimes of the owing impl's header must be equal, and nothing says so.
trait Tk<A> {}
struct S;
impl<'x> Tk<(&'x u32, &'x u32)> for S {}
trait Sup<A>: Tk<A> {}
impl<'a, 'b> Sup<(&'a u32, &'b u32)> for S {}
fn main() {
    println!("x");
}
