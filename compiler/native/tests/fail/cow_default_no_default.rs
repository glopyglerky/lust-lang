// expect: L0612
// rustc E0277: Cow<D> has no Default because D (its Owned) has none.
use std::borrow::Cow;

#[derive(Clone)]
struct D {
    n: i32,
}

fn main() {
    let c: Cow<D> = Cow::default();
    println!("{}", c.n);
}
