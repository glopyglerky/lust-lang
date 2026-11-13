// expect: L0312
// ARCH 2.8, rustc E0506: the dyn value holds `&x` (through `&&u8`), so `x` stays borrowed.
trait Tr { fn m(&self) -> u8; }
impl Tr for &u8 { fn m(&self) -> u8 { **self } }
fn main() { let mut x = 5u8; let r = &x; let d: &dyn Tr = &r; x = 6; println!("{}", d.m()); }
