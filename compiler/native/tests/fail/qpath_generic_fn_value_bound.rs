// expect: L0612
// rustc E0277: the impl needs NoDefault: Default.
trait Tr { fn mk(x: u8) -> Self; }
struct W<T>(T, u8);
struct NoDefault;
impl<T: Default> Tr for W<T> { fn mk(x: u8) -> Self { W(T::default(), x) } }
fn main() { let f = <W<NoDefault> as Tr>::mk; let w = f(3); println!("{}", w.1); }
