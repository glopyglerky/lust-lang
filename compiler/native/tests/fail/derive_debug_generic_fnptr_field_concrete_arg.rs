// expect: L0112
// rustc E0277: `Out`'s derived Debug needs `G<Leaf>: Debug`. The
// field `fn() -> T` is Debug for every T, but the derive on `G<T>`
// bounds `T: Debug`, so `Leaf` must be Debug too.
struct Leaf(u8);
#[derive(Debug)]
struct G<T>(fn() -> T, u8);
#[derive(Debug)]
struct Out {
    a: G<Leaf>,
}
fn mk() -> Leaf {
    Leaf(4)
}
fn main() {
    let o = Out { a: G(mk, 1) };
    println!("{} {}", (o.a.0)().0, o.a.1);
}
