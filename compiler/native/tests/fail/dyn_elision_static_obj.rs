// expect: L0202
// rustc E0106: the written `'static` object lifetime is a second input lifetime next to the reference's.
trait T { fn f(&self) -> u8; }
struct S(u8);
impl T for S { fn f(&self) -> u8 { self.0 } }
static K: u8 = 9;
fn g(d: &(dyn T + 'static)) -> &u8 { let _ = d.f(); &K }
fn main() { let s = S(3); println!("{}", g(&s)); }
