// expect: L0110
// rustc E0308: <[u16; 4] as Array>::Item is u16.
pub unsafe trait Array { type Item; fn size() -> usize; }
unsafe impl<T, const N: usize> Array for [T; N] { type Item = T; fn size() -> usize { N } }
struct P<A: Array> { v: Vec<A::Item> }
impl<A: Array> P<A> { fn push(&mut self, x: A::Item) { self.v.push(x); } }
fn main() { let mut p: P<[u16; 4]> = P { v: Vec::new() }; p.push(7u8); println!("{}", p.v.len()); }
