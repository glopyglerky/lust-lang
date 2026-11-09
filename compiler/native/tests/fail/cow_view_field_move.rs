// expect: L0305
// rustc E0507: move out of a field through a Cow deref.
use std::borrow::Cow;

#[derive(Clone)]
struct D {
    s: String,
}

fn main() {
    let d = D { s: "x".to_string() };
    let c: Cow<D> = Cow::Borrowed(&d);
    let s = c.s;
    println!("{}", s);
}
