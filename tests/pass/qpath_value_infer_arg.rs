// A `_` trait argument in a qualified path used as a value (not called)
// is a body inference variable, as in a qualified call (it panicked).
trait B<X> { fn b(&self, x: X) -> X; const K: u32; }
struct A;
impl B<u16> for A { fn b(&self, x: u16) -> u16 { x + 2 } const K: u32 = 9; }
fn main() {
    let f = <A as B<_>>::b;
    println!("{} {}", f(&A, 5), <A as B<_>>::K);
}
