// ARCH 2.8: a `&T` blanket impl: object methods win at `&dyn` by value, and `&dyn Tr` itself unsizes.
trait Tr { fn m(&self) -> u32; }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
impl<T: Tr + ?Sized> Tr for &T { fn m(&self) -> u32 { (**self).m() + 100 } }
fn main() {
    let a = A(4);
    let r = &a;
    let d: &dyn Tr = &r;
    let e: &dyn Tr = &a;
    let f: &dyn Tr = &e;
    println!("{} {} {}", d.m(), e.m(), f.m());
}
