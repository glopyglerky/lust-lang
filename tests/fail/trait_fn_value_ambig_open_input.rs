// expect: L0720
// rustc E0034: `Tr2<X>` has two impls for `S`, so its selection stays
// open, but it still applies next to `Tr`: `S::f` is ambiguous.
trait Tr { fn f(&self) -> i32; }
impl Tr for S { fn f(&self) -> i32 { self.0 * 2 } }
trait Tr2<X> { fn f(&self) -> i32 { 307 } }
impl Tr2<u8> for S {}
impl Tr2<u16> for S {}
struct S(i32);
fn main() { let g = S::f; println!("{}", g(&S(3))); }
