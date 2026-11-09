// expect: L0612
// ARCH 2.8, rustc E0277: `Ok(&u)` under a `Result<&dyn T, _>` expectation needs `U: T`.
trait T { fn v(&self) -> u8; }
struct S(u8);
struct U(u8);
impl T for S { fn v(&self) -> u8 { self.0 } }
fn main() { let u = U(1); let r: Result<&dyn T, u8> = Ok(&u); if let Ok(d) = r { println!("{}", d.v()); } }
