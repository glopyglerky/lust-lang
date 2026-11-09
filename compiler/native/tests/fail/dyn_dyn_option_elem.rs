// expect: L0110
// ARCH 2.8, rustc E0277: a generic param without `?Sized` cannot take `dyn Tr`.
trait Tr { fn m(&self) -> u8; }
struct W<T> { t: T }
fn f(_x: &W<dyn Tr>) {}
fn main() {}
