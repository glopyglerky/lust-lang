// expect: L0720
// rustc E0034: the call form of trait_fn_value_ambig_open_input.
trait Tr { fn f(&self) -> i32; }
impl Tr for S { fn f(&self) -> i32 { self.0 * 2 } }
trait Tr2<X> { fn f(&self) -> i32 { 307 } }
impl Tr2<u8> for S {}
impl Tr2<u16> for S {}
struct S(i32);
fn main() { println!("{}", S::f(&S(4))); }
