// expect: L0631
// rustc E0493: the D temporary is destroyed during const evaluation.
struct D(u8);
impl Drop for D { fn drop(&mut self) {} }
const fn mk(x: u8) -> D { D(x) }
const C: u8 = mk(4).0;
fn main() { println!("{}", C); }
