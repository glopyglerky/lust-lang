// expect: L0705
// ARCH 2.8, rustc E0521: a qualified call of a `where Self: 'static` method needs a `'static` dyn value.
trait T { fn f(&self) -> u8; fn h(&self) -> u8 where Self: 'static { 8 } }
struct S(u8);
impl T for S { fn f(&self) -> u8 { self.0 } }
fn call(d: &dyn T) -> u8 { <dyn T as T>::h(d) }
fn main() { let s = S(3); println!("{}", call(&s)); }
