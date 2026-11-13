// expect: L0308
// rustc E0596: Cow has no DerefMut: sort through the view.
use std::borrow::Cow;

fn main() {
    let v = vec![2u8, 1];
    let mut c: Cow<[u8]> = Cow::Borrowed(&v);
    c.sort();
}
