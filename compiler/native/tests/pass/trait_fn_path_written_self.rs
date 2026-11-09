// A trait fn through a written Self type selects the impl for that
// type: a turbofish on the type, `Self` in an impl, or an alias.
struct M<T>(T);
trait Mk {
    fn mk() -> Self;
}
impl Mk for M<u16> {
    fn mk() -> Self {
        M(16)
    }
}
impl Mk for M<u8> {
    fn mk() -> Self {
        M(8)
    }
}
impl M<u16> {
    fn again() -> Self {
        Self::mk()
    }
}
#[derive(Default)]
struct D<K, V> {
    k: Vec<K>,
    v: Vec<V>,
}
type A = M<u8>;
fn main() {
    let b = M::<u8>::mk();
    let c = M::<u16>::again();
    let d = A::mk();
    println!("{} {} {}", b.0, c.0, d.0);
    let e = D::<u8, i64>::default();
    println!("{} {}", e.k.len(), e.v.len());
}
