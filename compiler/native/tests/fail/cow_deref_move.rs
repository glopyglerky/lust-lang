// expect: L0305
// rustc E0507: move out of a Cow deref.
use std::borrow::Cow;

fn main() {
    let v = vec![1u8];
    let c: Cow<Vec<u8>> = Cow::Borrowed(&v);
    let w = *c;
    println!("{w:?}");
}
