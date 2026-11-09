// expect: L0308
// rustc E0596: Cow has no DerefMut, so a `&mut self` method of `B`
// cannot be called through the view.
use std::borrow::Cow;

#[derive(Clone)]
struct D {
    n: i32,
}
impl D {
    fn bump(&mut self) {
        self.n += 1;
    }
}

fn main() {
    let d = D { n: 1 };
    let mut c: Cow<D> = Cow::Borrowed(&d);
    c.bump();
    println!("{}", c.n);
}
