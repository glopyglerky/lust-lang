// expect: L0113
// rustc E0277: Cow<str> has two Add impls (&str and Cow<str>), so the right operand is not coerced.
use std::borrow::Cow;
fn main() {
    let t = String::from("t");
    let c: Cow<str> = Cow::Borrowed("c");
    let s = c + &t;
    println!("{s}");
}
