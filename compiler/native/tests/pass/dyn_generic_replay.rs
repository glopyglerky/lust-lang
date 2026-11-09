// ARCH 2.8, 4.1: a generic body resolves `x.m()`, `T::m(x)`, and `Tr::m(x)` to the trait method once;
// instantiated at `dyn Tr` they call through the vtable, never the `impl dyn Tr` method of that name.
trait Base { fn base(&self) -> u8 { 7 } }
trait Tr: Base { fn m(&self) -> u8; }
impl dyn Tr { fn m(&self) -> u8 { 100 } }
struct A;
impl Base for A {}
impl Tr for A { fn m(&self) -> u8 { 1 } }
fn call<T: Tr + ?Sized>(x: &T) -> u8 { T::m(x) }
fn call2<T: Tr + ?Sized>(x: &T) -> u8 { x.m() + x.base() }
fn call3<T: Tr + ?Sized>(x: &T) -> u8 { Tr::m(x) + <T as Tr>::m(x) }
fn main() {
    let d: &dyn Tr = &A;
    println!("{} {} {} {}", call(d), call2(d), call3(d), <dyn Tr as Tr>::m(d));
    println!("{} {}", call(&A), call2(&A));
}
