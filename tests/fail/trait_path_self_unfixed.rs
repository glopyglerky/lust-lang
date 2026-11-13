// expect: L0111
// rustc E0790: nothing fixes Self.
trait Mk { fn mk(n: u8) -> Self; }
struct S { n: u8 }
impl Mk for S { fn mk(n: u8) -> S { S { n } } }
fn main() { let _s = Mk::mk(5); println!("x"); }
