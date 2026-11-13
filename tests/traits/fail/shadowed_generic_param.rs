// rustc-error: E0403
// lust-expect: L0106
struct W<T>(T);
impl<T> W<T> { fn f<T>(&self, t: T) -> u8 { 1 } }
fn main() { println!("x"); }
