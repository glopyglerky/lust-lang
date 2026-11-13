// expect: L0306
// ARCH 2.8, rustc E0597: `dyn Tr + 'static` needs a 'static concrete type; `&x` is not.
trait Tr { fn m(&self) -> u8; }
impl Tr for &u8 { fn m(&self) -> u8 { **self } }
fn keep(d: &(dyn Tr + 'static)) -> u8 { d.m() }
fn main() { let x = 5u8; let r = &x; println!("{}", keep(&r)); }
