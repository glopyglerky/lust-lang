// expect: L0307
// ARCH 2.8, rustc E0515: the returned `&dyn` borrows a local, not the parameter.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
fn mk(_x: &u8) -> &dyn Tr { let s = S; &s }
fn main() { let y = 1; println!("{}", mk(&y).m()); }
