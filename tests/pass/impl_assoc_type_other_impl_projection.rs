// An impl's assoc type bound to another impl's projection
// (`type O = <A as Tr>::O;`) is that impl's type, not a cycle.
trait Tr {
    type O;
    fn mk() -> Self::O;
}
struct A;
struct B;
impl Tr for A {
    type O = u8;
    fn mk() -> u8 {
        200
    }
}
impl Tr for B {
    type O = <A as Tr>::O;
    fn mk() -> u8 {
        3
    }
}
fn main() {
    println!("{} {}", A::mk(), B::mk());
}
