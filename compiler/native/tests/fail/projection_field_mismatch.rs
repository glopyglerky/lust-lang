// expect: L0110
// rustc E0308: a projection field takes the impl's item type.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct P<A: Array> { v: Vec<A::Item>, n: A::Item }
fn main() { let p: P<[u16; 4]> = P { v: Vec::new(), n: 5i8 }; println!("{}", p.v.len()); }
