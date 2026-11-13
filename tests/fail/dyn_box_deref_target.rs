// expect: L0612
// rustc E0277: a `&dyn Tr` target commits to unsizing, so `&Box<S>`
// needs `Box<S>: Tr`; the `Box` deref coercion never applies there.
trait Tr { fn read(&self) -> u32; }
struct S { n: u32 }
impl Tr for S { fn read(&self) -> u32 { self.n } }
fn main() { let b = Box::new(S { n: 5 }); let d: &dyn Tr = &b; println!("{}", d.read()); }
