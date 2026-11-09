// expect: L0705
// ARCH 2.8, rustc E0521: `b` needs `Self: 'static`; the object lifetime of a `&mut dyn T` param is the caller's.
trait B { fn b(&self) -> u8 where Self: 'static { 8 } }
trait T: B { fn f(&self) -> u8; }
struct S(u8);
impl B for S {}
impl T for S { fn f(&self) -> u8 { self.0 } }
fn call(d: &mut dyn T) -> u8 { d.b() + d.f() }
fn main() { let mut s = S(3); println!("{}", call(&mut s)); }
