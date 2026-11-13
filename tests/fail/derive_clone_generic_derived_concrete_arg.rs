// expect: L0112
// rustc E0277: `Out<U>`'s derived Clone needs `G<Leaf>: Clone`, so
// `Leaf: Clone` under the derive's bound on `G<T>`, though the field
// `Option<fn() -> T>` itself is Clone for every T.
#[derive(Clone)]
struct G<T> {
    f: Option<fn() -> T>,
}
struct Leaf;
#[derive(Clone)]
struct Out<U> {
    a: Option<Box<G<Leaf>>>,
    u: U,
}
fn main() {
    let o = Out { a: Some(Box::new(G { f: None })), u: 4u8 };
    println!("{} {}", o.u, o.a.is_some());
}
