// expect: L0110
// rustc E0308: reading a projection field gives the item type.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct P<A: Array> { n: A::Item }
fn main() { let p: P<[u16; 4]> = P { n: 5 }; let x: u8 = p.n; println!("{}", x); }
