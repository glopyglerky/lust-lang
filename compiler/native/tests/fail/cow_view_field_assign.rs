// expect: L0309
// rustc E0594: assignment to a field through a Cow (no DerefMut).
use std::borrow::Cow;

#[derive(Clone)]
struct D {
    n: i32,
}

fn main() {
    let d = D { n: 1 };
    let mut c: Cow<D> = Cow::Borrowed(&d);
    c.n = 5;
    println!("{}", c.n);
}
