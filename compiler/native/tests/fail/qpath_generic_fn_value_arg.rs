// expect: L0110
// rustc E0308: the fn value takes a u8.
trait Tr { fn mk(x: u8) -> Self; }
struct W<T>(T, u8);
impl<T: Default> Tr for W<T> { fn mk(x: u8) -> Self { W(T::default(), x) } }
fn main() { let f = <W<u16> as Tr>::mk; let w = f("no"); println!("{}", w.1); }
