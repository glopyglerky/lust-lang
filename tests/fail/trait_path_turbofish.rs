// expect: L0612
// rustc E0277: the written trait argument of `B::<u8>::b` is part of the trait ref.
trait B<X> { fn b(&self, x: X) -> u32; }
struct A;
impl B<u16> for A { fn b(&self, x: u16) -> u32 { x as u32 + 2 } }
fn main() { println!("{}", B::<u8>::b(&A, 1)); }
