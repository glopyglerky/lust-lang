// rustc-error: E0277
// lust-expect: L0612
trait Tr {}
impl Tr for u8 {}
struct W<T: Tr>(T);
fn f<T>(w: W<T>) -> u8 { 1 }
fn main() { println!("{}", f(W(5u8))); }
