// A derive over a field whose written generic impl bounds a projection
// (smallvec's `impl<A: Array> Clone for SmallVec<A> where A::Item: Clone`;
// ts-rust checker/checker_p10.rs CallState).
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
struct SV<A: Array> { v: Vec<A::Item> }
impl<A: Array> Clone for SV<A> where A::Item: Clone { fn clone(&self) -> Self { SV { v: self.v.clone() } } }
impl<A: Array> std::fmt::Debug for SV<A> where A::Item: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "SV{:?}", self.v) }
}
#[derive(Clone, Debug, PartialEq)]
struct N(u32);
#[derive(Clone, Debug)]
struct St { n: N, a: SV<[N; 4]>, b: SV<[String; 4]> }
fn main() {
    let s = St { n: N(1), a: SV { v: vec![N(2), N(3)] }, b: SV { v: vec!["x".to_string()] } };
    let t = s.clone();
    println!("{:?} {:?}", t, s.a.v.len());
}
