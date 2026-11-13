// expect: L0308
// rustc E0596: `&mut` through a Cow index (no DerefMut).
use std::borrow::Cow;

fn main() {
    let v = vec![1u8, 2];
    let mut c: Cow<[u8]> = Cow::Borrowed(&v);
    c[0] = 9;
    println!("{}", c[0]);
}
