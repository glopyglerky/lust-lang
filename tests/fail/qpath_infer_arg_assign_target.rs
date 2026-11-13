// expect: L0121
// rustc E0070: a qualified path with a `_` trait argument as an
// assignment target (lust panicked on its unreplaced `_` marker).
trait B<X> { fn b(&self, x: X) -> X; }
struct A;
impl B<u16> for A { fn b(&self, x: u16) -> u16 { x + 2 } }
fn main() { <A as B<_>>::b = (&A, 5); }
