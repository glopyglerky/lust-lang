// expect: L0113
// rustc E0277: only Cow<str> has Add; Cow<[u8]> does not.
use std::borrow::Cow;

fn main() {
    let a: Cow<[u8]> = Cow::Borrowed(&[1u8]);
    let b = a + "x";
    println!("{:?}", b);
}
