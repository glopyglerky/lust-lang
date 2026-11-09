// The impl proves the supertrait for every lifetime. A where
// clause at 'static that cannot prove the 'a case does not
// block it; rustc uses the impl.
trait Tk<A> {}
trait Sup<A>: Tk<A> {}
struct S;
impl<T> Tk<T> for S {}
impl<'a> Sup<&'a u32> for S where S: Tk<&'static u32> {}
fn want<T: Sup<&'static u32>>(_t: &T) -> u8 { 4 }
fn main() {
    println!("{}", want(&S));
}
