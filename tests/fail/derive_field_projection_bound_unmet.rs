// expect: L0112
// rustc E0277: `SV<[Z; 4]>: Clone` needs `Z: Clone`.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct SV<A: Array> { v: Vec<A::Item> }
impl<A: Array> Clone for SV<A> where A::Item: Clone { fn clone(&self) -> Self { SV { v: self.v.clone() } } }
struct Z(u8);
#[derive(Clone)]
struct St { a: SV<[Z; 4]> }
fn main() { let _ = St { a: SV { v: vec![Z(1)] } }; }
