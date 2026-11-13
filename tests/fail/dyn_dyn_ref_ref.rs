// expect: L0612
// ARCH 2.8, rustc E0277: `&&dyn Tr` to `&dyn Tr` unsizes `&dyn Tr`, which has no impl; no deref coercion.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
fn f(d: &dyn Tr) -> u8 { d.m() }
fn main() { let s = S; let d: &dyn Tr = &s; println!("{}", f(&d)); }
