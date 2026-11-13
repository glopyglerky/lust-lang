// A `_` trait argument in a qualified path is a body inference variable (it was an internal error).
trait B<X> { fn b(&self, x: X) -> X; }
struct A;
impl B<u16> for A { fn b(&self, x: u16) -> u16 { x + 2 } }
fn main() {
    let d: &dyn B<u16> = &A;
    println!("{} {}", <A as B<_>>::b(&A, 5), <dyn B<u16> as B<_>>::b(d, 6));
}
