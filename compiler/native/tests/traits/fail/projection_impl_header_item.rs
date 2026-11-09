// rustc-error: E0271
// lust-expect: L0612
// ARCH 2.4/2.5: an impl header projection normalizes before it
// matches: `SmallVec<[u8; 4]>` sinks `u8`, not `u16`.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct SmallVec<A: Array> { data: Vec<A::Item> }
trait Sink<X> { fn put(&mut self, x: X); }
impl<A: Array> Sink<A::Item> for SmallVec<A> { fn put(&mut self, x: A::Item) { self.data.push(x); } }
fn fill<S: Sink<u16>>(s: &mut S) { s.put(1); }
fn main() { let mut v: SmallVec<[u8; 4]> = SmallVec { data: Vec::new() }; fill(&mut v); println!("{}", v.data.len()); }
