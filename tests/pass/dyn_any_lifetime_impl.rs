// ARCH 2.8: `impl dyn Tr + '_` and `impl X for dyn Tr + '_` take any object
// lifetime, so a `&dyn Tr` param can use them.
trait Tr { fn m(&self) -> u32; }
impl dyn Tr + '_ { fn h(&self) -> u32 { self.m() + 1 } }
trait Describe { fn describe(&self) -> u32; }
impl Describe for dyn Tr + '_ { fn describe(&self) -> u32 { self.h() + 100 } }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
fn show<T: Describe + ?Sized>(x: &T) -> u32 { x.describe() }
fn f(d: &dyn Tr) -> u32 { d.h() + d.describe() + show(d) }
fn main() { println!("{}", f(&A(1))); }
