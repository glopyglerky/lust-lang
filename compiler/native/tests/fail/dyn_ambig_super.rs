// expect: L0720
// ARCH 2.8, rustc E0034: a supertrait and the principal declare the same method name.
trait A { fn m(&self) -> u8; }
trait B: A { fn m(&self) -> u8; }
struct S;
impl A for S { fn m(&self) -> u8 { 1 } }
impl B for S { fn m(&self) -> u8 { 2 } }
fn main() { let s = S; let d: &dyn B = &s; println!("{}", d.m()); }
