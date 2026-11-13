// A trait lifetime param that nothing uses, and a qualified call naming it.
trait T<'a> { fn f(&self) -> u8 { 1 } }
struct S;
impl<'a> T<'a> for S {}
fn main() { println!("{}", <S as T<'static>>::f(&S)); }
