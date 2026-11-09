// expect: L0301
// rustc E0382: the box moved into the first call.
trait A { fn a(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
fn take(b: Box<dyn A>) -> u8 { b.a() }
fn main() { let b: Box<dyn A> = Box::new(S); take(b); take(b); }
