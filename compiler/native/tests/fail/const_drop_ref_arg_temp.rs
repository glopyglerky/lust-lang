// expect: L0631
// rustc E0493: the borrowed D temporary is destroyed in the const item.
struct D(u8);
impl Drop for D { fn drop(&mut self) {} }
const fn get(d: &D) -> u8 { d.0 }
const C: u8 = get(&D(5));
fn main() { println!("{}", C); }
