// expect: L0612
// rustc E0277: `S` does not implement `A`.
trait A { fn a(&self) -> u8; }
struct S;
fn main() { let b: Box<dyn A> = Box::new(S); println!("{}", b.a()); }
