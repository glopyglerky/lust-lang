// expect: L0110
// rustc E0308: mk takes a u8.
trait Mk { fn mk(n: u8) -> Self; }
struct S { n: u8 }
impl Mk for S { fn mk(n: u8) -> S { S { n } } }
fn main() { let s: S = Mk::mk(300u16); println!("{}", s.n); }
