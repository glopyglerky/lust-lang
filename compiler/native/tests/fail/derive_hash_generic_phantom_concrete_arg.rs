// expect: L0110
// rustc E0277: a derived Hash on `G<T>` bounds `T: Hash` even when
// only a PhantomData field names T, so `Out` cannot derive Hash over
// `Option<G<Leaf>>`.
use std::marker::PhantomData;
struct Leaf;
#[derive(Hash)]
struct G<T> {
    p: PhantomData<T>,
    n: u8,
}
#[derive(Hash)]
struct Out {
    a: Option<G<Leaf>>,
}
fn main() {
    let o = Out { a: Some(G { p: PhantomData, n: 3 }) };
    println!("{}", o.a.map_or(0, |g| g.n));
}
