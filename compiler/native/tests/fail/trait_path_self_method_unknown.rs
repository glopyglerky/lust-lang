// expect: L0610
// rustc E0282: a method call needs the receiver type known.
trait Mk { fn mk(n: u8) -> Self; }
struct S { n: u8 }
impl S { fn show(&self) -> u8 { self.n } }
impl Mk for S { fn mk(n: u8) -> S { S { n } } }
fn main() { let x = Mk::mk(1); println!("{}", x.show()); let _y: S = x; }
