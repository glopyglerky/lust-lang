// The implicit `T: Sized` of `impl<T> Tr for T` keeps it off unsized types,
// so impls for `str`, `[u8]`, and `dyn Fn()` do not overlap it (rustc accepts).
trait Tr { fn f(&self) -> i32 { 0 } }
impl<T> Tr for T {}
impl Tr for str { fn f(&self) -> i32 { 1 } }
impl Tr for [u8] { fn f(&self) -> i32 { 2 } }
trait Cl { fn c(&self) -> i32 { 0 } }
impl<T: Clone> Cl for T {}
impl Cl for str { fn c(&self) -> i32 { 3 } }
fn main() {
    let b: &[u8] = &[1, 2];
    println!("{} {} {} {} {}", 5.f(), "x".f(), b.f(), 5.c(), "y".c());
}
