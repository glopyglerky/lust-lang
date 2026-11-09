// expect: L0612
// rustc E0277: `B::<u8>::b` names `B<u8>`; a `dyn B<u16>` receiver does not implement it.
trait B<X> { fn b(&self, x: X) -> u32; }
struct A;
impl B<u16> for A { fn b(&self, x: u16) -> u32 { x as u32 + 2 } }
fn main() { let d: &dyn B<u16> = &A; println!("{}", B::<u8>::b(d, 1)); }
