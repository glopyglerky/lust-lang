// expect: L0705
// rustc E0521: the impl picked for `W<&dyn T>` needs `&dyn T: 'static`.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
trait Tr { fn tr(&self) -> u32; }
struct W<X>(X);
impl<X: 'static> Tr for W<X> { fn tr(&self) -> u32 { 1 } }
fn bad(d: &dyn T) -> u32 { W(d).tr() }
fn main() {}
