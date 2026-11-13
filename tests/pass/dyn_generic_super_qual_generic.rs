// ARCH 2.8: in a generic body, `<dyn T<Y> as B<Y>>` and `<dyn T<Y> as B<u16>>` are two supertrait refs of one trait.
trait B<X> { fn b(&self, x: X) -> X; }
trait T<Y>: B<Y> + B<u16> { fn g(&self) -> u8; }
struct A;
impl B<u8> for A { fn b(&self, x: u8) -> u8 { x + 1 } }
impl B<u16> for A { fn b(&self, x: u16) -> u16 { x + 2 } }
impl T<u8> for A { fn g(&self) -> u8 { 3 } }
fn f<Y: Copy>(d: &dyn T<Y>, y: Y) -> (Y, u16) { (<dyn T<Y> as B<Y>>::b(d, y), <dyn T<Y> as B<u16>>::b(d, 10)) }
fn main() { let r = f(&A, 1u8); println!("{} {}", r.0, r.1); }
