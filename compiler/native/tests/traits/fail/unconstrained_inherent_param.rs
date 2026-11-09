// rustc-error: E0207
// lust-expect: L0110
struct S;
impl<T> S { fn v(&self) -> u8 { 1 } }
fn main() { println!("{}", S.v()); }
