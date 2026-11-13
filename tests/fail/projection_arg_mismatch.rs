// expect: L0110
// rustc E0308: the param is the impl's item type, u16.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct P<A: Array> { v: Vec<A::Item> }
impl<A: Array> P<A> { fn push(&mut self, x: A::Item) { self.v.push(x); } fn pop(&mut self) -> Option<A::Item> { self.v.pop() } }
fn main() { let mut p: P<[u16; 4]> = P { v: Vec::new() }; p.push("no"); println!("{}", p.v.len()); }
