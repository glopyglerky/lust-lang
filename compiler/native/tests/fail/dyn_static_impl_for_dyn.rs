// expect: L0705
// ARCH 2.8, rustc E0521: `impl Describe for dyn Tr` needs `dyn Tr + 'static`.
trait Tr { fn m(&self) -> u32; }
trait Describe { fn describe(&self) -> u32; }
impl Describe for dyn Tr { fn describe(&self) -> u32 { self.m() + 100 } }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
fn show<T: Describe + ?Sized>(x: &T) -> u32 { x.describe() }
fn f(d: &dyn Tr) -> u32 { d.describe() }
fn main() { println!("{}", f(&A(1))); }
