// expect: L0612
// rustc E0277: the later use makes Self = T, which has no Mk impl.
trait Mk { fn mk(n: u8) -> Self; }
struct S { n: u8 }
struct T;
impl Mk for S { fn mk(n: u8) -> S { S { n } } }
fn main() { let x = Mk::mk(5); let _t: T = x; println!("x"); }
