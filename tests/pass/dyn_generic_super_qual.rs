// ARCH 2.8: a generic trait can be a supertrait twice (`B<u8> + B<u16>`). A qualified call or a trait
// path with written args picks the vtable of that one supertrait ref; upcasts pick it by the target type.
trait B<X> { fn b(&self, x: X) -> X; }
trait T: B<u8> + B<u16> { fn g(&self) -> u8; }
struct A;
impl B<u8> for A { fn b(&self, x: u8) -> u8 { x + 1 } }
impl B<u16> for A { fn b(&self, x: u16) -> u16 { x + 2 } }
impl T for A { fn g(&self) -> u8 { 3 } }
fn main() {
    let d: &dyn T = &A;
    println!("{} {}", <dyn T as B<u16>>::b(d, 5), <dyn T as B<u8>>::b(d, 5));
    println!("{} {}", B::<u16>::b(d, 10), B::<u8>::b(d, 10));
    let b8: &dyn B<u8> = d;
    let b16: &dyn B<u16> = d;
    println!("{} {} {}", b8.b(1), b16.b(1), d.g());
    println!("{} {}", <A as B<u16>>::b(&A, 7), B::<u8>::b(&A, 7));
}
