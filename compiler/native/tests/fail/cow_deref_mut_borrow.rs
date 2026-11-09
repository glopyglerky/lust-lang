// expect: L0308
// rustc E0596: &mut through a Cow deref.
use std::borrow::Cow;

fn main() {
    let v = vec![1u8];
    let mut c: Cow<[u8]> = Cow::Borrowed(&v);
    let r = &mut *c;
    r[0] = 2;
}
