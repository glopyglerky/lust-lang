// expect: L0110
// rustc E0308: in a generic body the item type is rigid, not an integer.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct P<A: Array> { v: Vec<A::Item> }
impl<A: Array> P<A> { fn push(&mut self, x: A::Item) { self.v.push(x); } fn pop(&mut self) -> Option<A::Item> { self.v.pop() } }
fn g<A: Array>(p: &mut P<A>) { p.push(5); }
fn main() { let mut p: P<[i32; 4]> = P { v: Vec::new() }; g(&mut p); println!("{}", p.v.len()); }
