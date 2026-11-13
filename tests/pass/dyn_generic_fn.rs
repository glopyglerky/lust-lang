// ARCH 2.8: a `?Sized` generic fn instantiated at `dyn Tr`, and a generic unsize.
trait Tr { fn m(&self) -> u32; }
struct A(u32);
impl Tr for A { fn m(&self) -> u32 { self.0 } }
fn call<T: Tr + ?Sized>(x: &T) -> u32 { x.m() + 1 }
fn twice<T: Tr + ?Sized>(x: &T) -> u32 { call(x) + call(x) }
fn erase<T: Tr>(x: &T) -> &dyn Tr { x }
fn main() {
    let a = A(5);
    let d: &dyn Tr = &a;
    println!("{} {} {}", call(d), call(&a), twice(d));
    println!("{}", erase(&a).m());
}
