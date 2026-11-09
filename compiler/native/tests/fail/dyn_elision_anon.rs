// expect: L0202
// rustc E0106: `&(dyn T + '_)` has two input lifetimes, so a returned reference has no single source.
trait T { fn f(&self) -> u8; }
struct S(u8);
impl T for S { fn f(&self) -> u8 { self.0 } }
fn g(d: &(dyn T + '_)) -> &(dyn T + '_) { d }
fn main() { let s = S(3); println!("{}", g(&s).f()); }
